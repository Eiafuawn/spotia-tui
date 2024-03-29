use std::{
    collections::HashMap,
    env, fs,
    io::{self, BufRead, BufReader, Write},
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use rspotify::model::SimplifiedPlaylist;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
    action::Action,
    app,
    config::{Config, KeyBindings},
    mode::Mode,
};

#[derive(Default)]
pub struct Home {
    mode: Mode,
    index: usize,
    offset: usize,
    dir: String,
    playlists: Vec<SimplifiedPlaylist>,
    menus: Vec<String>,
    dirs: Vec<String>,
    display_list_len: usize,
    key_input: String,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl Home {
    pub fn new(playlists: Vec<SimplifiedPlaylist>) -> Self {
        let menus = vec![
            "Download Playlist".to_string(),
            "Manage Downloads".to_string(),
            "Settings".to_string(),
            "Quit".to_string(),
        ];
        let display_list_len = menus.len();
        Self {
            mode: Mode::Input,
            menus,
            playlists,
            display_list_len,
            key_input: env::var("HOME").unwrap_or("".to_string()),
            ..Default::default()
        }
    }
    /// Moves the selection cursor up.
    pub fn move_up(&mut self) {
        if self.index > 0 {
            self.index -= 1;
            if self.offset > 0 {
                self.offset -= 1;
            }
        }
    }

    /// Moves the selection cursor down.
    pub fn move_down(&mut self) {
        if self.index < self.display_list_len - 1 {
            self.index += 1;
            if self.index - self.offset > 5 {
                self.offset += 1;
            }
        }
    }

    pub fn enter_downloader(&mut self) {
        self.index = 0;
        self.offset = 0;
        self.mode = Mode::Downloader;
        self.display_list_len = self.playlists.len();
    }

    fn enter_manager(&mut self, dirs: Vec<String>) {
        self.index = 0;
        self.offset = 0;
        self.dirs = dirs;
        self.mode = Mode::Manager;
        self.display_list_len = self.dirs.len();
    }

    fn enter_home(&mut self) {
        self.mode = Mode::Home;
        self.index = 0;
        self.offset = 0;
        self.display_list_len = self.menus.len();
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
            Action::MoveUp => self.move_up(),
            Action::MoveDown => self.move_down(),
            Action::EnterEditing => self.mode = Mode::Input,
            Action::EnterDownloader => self.enter_downloader(),
            Action::SelectFolder(_) => self.mode = Mode::Home,
            Action::EnterManager => self.mode = Mode::Manager,
            Action::GetDirs(dirs) => self.enter_manager(dirs),
            Action::DownloadFinished => self.mode = Mode::Waiting,
            Action::BackHome => self.enter_home(),
            Action::QuitEditing => {
                self.mode = Mode::Home;
                self.key_input = env::var("HOME").unwrap_or("".to_string())
            }
            Action::SelectPlaylist(_) | Action::SelectActivePlaylist(_) => {
                self.mode = Mode::Downloading
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match self.mode {
            Mode::Input => match key.code {
                KeyCode::Char(value) => {
                    self.key_input.push(value);
                    Action::Resume
                }
                KeyCode::Esc => Action::QuitEditing,
                KeyCode::Backspace => {
                    self.key_input.pop();
                    Action::Resume
                }
                KeyCode::Enter => {
                    self.mode = Mode::Home;
                    Action::SelectFolder(self.key_input.clone())
                }
                _ => Action::Resume,
            },
            Mode::Home => match key.code {
                KeyCode::Up => Action::MoveUp,
                KeyCode::Down => Action::MoveDown,
                KeyCode::Enter => match self.index {
                    0 => Action::EnterDownloader,
                    1 => Action::EnterManager,
                    2 => Action::Resume,
                    3 => Action::Quit,
                    _ => Action::Resume,
                },
                _ => Action::Resume,
            },
            Mode::Downloader => match key.code {
                KeyCode::Enter => Action::SelectPlaylist(self.index),
                _ => Action::Resume,
            },
            Mode::Manager => match key.code {
                KeyCode::Enter => Action::SelectActivePlaylist(self.index),
                _ => Action::Resume,
            },
            Mode::Waiting => match key.code {
                KeyCode::Enter => Action::BackHome,
                _ => Action::Resume,
            },
            _ => Action::Resume,
        };
        Ok(Some(action))
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        match self.mode {
            Mode::Downloader => {
                let playlists: Vec<ListItem> = self
                    .playlists
                    .iter()
                    .skip(self.offset)
                    .enumerate()
                    .map(|(i, item)| {
                        let style = if i == self.index - self.offset {
                            Style::default().bg(Color::Yellow).fg(Color::Black)
                        } else {
                            Style::default()
                        };
                        ListItem::new(item.name.clone()).style(style)
                    })
                    .collect();
                let playlist_menu = List::new(playlists.clone())
                    .block(Block::default().borders(Borders::ALL))
                    .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

                f.render_widget(playlist_menu, area);
            }
            Mode::Input => {
                f.render_widget(Clear, area);
                let popup_block = Block::default()
                    .title("Choose your folder")
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::Gray));

                let popup = Paragraph::new(self.key_input.clone())
                    .style(Style::default().bg(Color::White).fg(Color::Black))
                    .block(popup_block);

                let center = centered_rect(60, 25, area);
                f.render_widget(popup, center);
            }
            Mode::Home => {
                let menus: Vec<ListItem> = self
                    .menus
                    .iter()
                    .skip(self.offset)
                    .enumerate()
                    .map(|(i, item)| {
                        let style = if i == self.index - self.offset {
                            Style::default().bg(Color::Yellow).fg(Color::Black)
                        } else {
                            Style::default()
                        };
                        ListItem::new(item.clone()).style(style)
                    })
                    .collect();
                let menu = List::new(menus.clone())
                    .block(Block::default().borders(Borders::ALL))
                    .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

                f.render_widget(menu, area)
            }
            Mode::Manager => {
                let dirs: Vec<ListItem> = self
                    .dirs
                    .iter()
                    .skip(self.offset)
                    .enumerate()
                    .map(|(i, item)| {
                        let style = if i == self.index - self.offset {
                            Style::default().bg(Color::Yellow).fg(Color::Black)
                        } else {
                            Style::default()
                        };
                        ListItem::new(item.clone()).style(style)
                    })
                    .collect();
                let dir_menu = List::new(dirs.clone())
                    .block(Block::default().borders(Borders::ALL))
                    .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));
                f.render_widget(dir_menu, area);
            }
            _ => {}
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
