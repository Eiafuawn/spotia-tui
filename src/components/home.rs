use std::{
    collections::HashMap, 
    time::Duration,
    fs, 
    env,
    io::{self, BufRead, BufReader, Write},
    path::Path, process::{Command, Stdio}
};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use rspotify::model::SimplifiedPlaylist;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
    action::Action, app, config::{Config, KeyBindings}, mode::Mode, spotify::Spotify
};

#[derive(Default)]
pub struct Home {
    mode: Mode,
    playlist_index: usize,
    offset: usize,
    spotify: Spotify,
    download_output: String,
    key_input: String,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl Home {
    pub fn new(spotify: Spotify) -> Self {
        Self{
            mode: Mode::Home,
            spotify,
            key_input: env::var("HOME")
                .unwrap_or("".to_string()),
            ..Default::default()
        }
    }
   /// Moves the selection cursor up.
    pub fn move_up(&mut self) {
        if self.playlist_index > 0 {
            self.playlist_index -= 1;
            if self.offset > 0 {
                self.offset -= 1;
                            
            }
        }
    }

    /// Moves the selection cursor down.
    pub fn move_down(&mut self) {
        if self.playlist_index < self.spotify.playlists.len() - 1 {
            self.playlist_index += 1;
            if self.playlist_index - self.offset > 5 {
                self.offset += 1;
            }
        }
    }

    //// Download the selected playlist
    pub fn select_playlist(&mut self) -> Result<()>{
        let url = self.spotify.get_playlist_url(self.playlist_index);
        let name = self.spotify.get_playlist_name(self.playlist_index)
            .replace(' ', "");
        let dir = self.key_input.clone() + "/" + &name;
        let dir_path = Path::new(&dir);

        if !dir_path.exists() {
            if let Err(err) = fs::create_dir_all(dir_path) {
                self.download_output.push_str(&format!("Error creating directory: {}", err));
                            
            } else {
                self.download_output.push_str(&format!("Directory {} created successfully!", dir));
                self.download_playlist(url, dir)?;
            }
        } else {
            self.download_output.push_str(&format!("Directory {} already exists!", dir));
            self.sync_playlist(dir_path)?;
        }

        self.mode = Mode::Home;
        Ok(())
    }

    //// Sync the selected playlist
    fn sync_playlist(&mut self, dir: &Path) -> Result<()> {
        self.download_output.push_str("Syncing playlist...");
        let stdout = Command::new("spotdl")
                    .args(["sync".to_string(), "save.spotdl".to_string()])
                    .current_dir(dir)
                    .stdout(Stdio::piped())  // Redirect stdout to a pipe
                    .spawn()?
                    .stdout
                    .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to execute command"))?;

        let reader = BufReader::new(stdout);

        reader.lines()
            .map_while(|line| line.ok())
            .for_each(|line| {
                self.download_output.push_str(&line);
                self.download_output.push('\n');
                            
            });

        self.key_input.clear();
        
        Ok(())
    }
    
    fn download_playlist(&mut self, url: String, dir: String) -> Result<()> {
        self.download_output.push_str("Downloading playlist...");
        let stdout = Command::new("spotdl")
            .args(["sync".to_string(), url, "--save-file".to_string(), "save.spotdl".to_string(), "--simple-tui".to_string()])
            .current_dir(dir)
            .stdout(Stdio::piped())  // Redirect stdout to a pipe
            .spawn()?
            .stdout
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to execute command"))?;

        let reader = BufReader::new(stdout);

        reader.lines()
            .map_while(|line| line.ok())
            .for_each(|line| {
                self.download_output.push_str(&line);
                self.download_output.push('\n');
                            
            });

        self.key_input.clear();
        
        Ok(())
    }

}

impl Component for Home {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {}
            Action::SelectPlaylist => {
                self.mode = Mode::Home;
                self.select_playlist()?
            },
            Action::MoveUp => self.move_up(),
            Action::MoveDown => self.move_down(),
            Action::EnterEditing => self.mode = Mode::SelectingDir,
            Action::QuitEditing => {
                self.mode = Mode::Home;
                self.key_input = env::var("HOME")
                    .unwrap_or("".to_string())
            },
            Action::Save => self.mode = Mode::Downloading,
            _ => {}
        }
        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>>  {
        let action = match self.mode {
        Mode::Home => match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Enter => Action::EnterEditing,
            KeyCode::Up | KeyCode::Char('k') => Action::MoveUp,
            KeyCode::Down | KeyCode::Char('j') => Action::MoveDown,
            _ => Action::Resume,
        },
        Mode::SelectingDir => match key.code {
            KeyCode::Char(value) => {
                self.key_input.push(value);
                Action::Resume // Assuming Resume is the default action for Editing mode
            }
            KeyCode::Esc => Action::QuitEditing, // Example: Action to quit editing mode
            KeyCode::Enter => Action::Save, // Example: Action to save changes
            _ => Action::Resume,
        },
        Mode::Downloading => Action::SelectPlaylist,
    };
    Ok(Some(action))    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let main_layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ],
        )
        .split(area);
        f.render_widget(
            Block::new()
                .borders(Borders::TOP)
                .title("Select a playlist to download"),
            main_layout[0],
        );
        f.render_widget(
            Block::new().borders(Borders::TOP).title("Status Bar"),
            main_layout[2],
        );

        let chunks = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .split(main_layout[1]);

        let playlists: Vec<ListItem> = self.spotify.playlists
            .iter()
            .skip(self.offset)
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.playlist_index - self.offset {
                    Style::default().bg(Color::Yellow).fg(Color::Black)
                } else {
                    Style::default()
                };
                ListItem::new(item.name.clone()).style(style)
            })
            .collect();

        let menu = List::new(playlists.clone())
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

        f.render_widget(menu, chunks[0]);

        let output = Paragraph::new(self.download_output.clone())
        .style(Style::default().fg(Color::Black).bg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("Output"));

        f.render_widget(output, chunks[1]);
        
        if self.mode == Mode::SelectingDir {
            f.render_widget(Clear, f.size());
            let popup_block = Block::default()
                .title("Choose your folder")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Gray));

            let popup = Paragraph::new(self.key_input.clone())
                .style(Style::default().bg(Color::White).fg(Color::Black))
                .block(popup_block);

            let area = centered_rect(60, 25, f.size());
            f.render_widget(popup, area);
        }


        Ok(())
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

