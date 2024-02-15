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
    playlist_index: usize,
    offset: usize,
    playlists: Vec<SimplifiedPlaylist>,
    key_input: String,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl Home {
    pub fn new(playlists: Vec<SimplifiedPlaylist>) -> Self {
        Self {
            mode: Mode::Home,
            playlists,
            key_input: env::var("HOME").unwrap_or("".to_string()),
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
        if self.playlist_index < self.playlists.len() - 1 {
            self.playlist_index += 1;
            if self.playlist_index - self.offset > 5 {
                self.offset += 1;
            }
        }
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
            Action::EnterEditing => self.mode = Mode::Downloader,
            Action::QuitEditing => {
                self.mode = Mode::Home;
                self.key_input = env::var("HOME").unwrap_or("".to_string())
            }
            Action::SelectPlaylist(_, _) => self.mode = Mode::Home,
            _ => {}
        }
        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match self.mode {
            Mode::Downloader => match key.code {
                KeyCode::Char(value) => {
                    self.key_input.push(value);
                    Action::Resume
                }
                KeyCode::Esc => Action::QuitEditing,
                KeyCode::Backspace => {
                    self.key_input.pop();
                    Action::Resume
                }
                KeyCode::Enter => Action::SelectPlaylist(
                    self.key_input.clone(), self.playlist_index
                    ),
                _ => Action::Resume,
            },
            _ => Action::Resume,
        };
        Ok(Some(action))
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let chunks = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .split(area);

        let playlists: Vec<ListItem> = self
            .playlists
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

        if self.mode == Mode::Downloader {
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
