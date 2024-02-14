use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use rspotify::model::SimplifiedPlaylist;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
    action::Action, app, config::{Config, KeyBindings}
};

#[derive(Default)]
pub struct Home {
    playlist_index: usize,
    offset: usize,
    playlists: Vec<SimplifiedPlaylist>,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl Home {
    pub fn new(list: Vec<SimplifiedPlaylist>) -> Self {
        Self{
            playlists: list,
            ..Default::default()
        }
    }
   /// Moves the selection cursor up.
    pub fn move_up(&mut self) {
        if self.playlist_index > 0 {
            self.playlist_index -= 1;
            if self.playlist_index < self.offset {
                // If moving up would make the selected index go above the visible range,
                // Adjust the offset to keep 5 items visible above the selected index
                self.offset = self.playlist_index.min(self.playlists.len() - 5);
            }
        }
    }

    /// Moves the selection cursor down.
    pub fn move_down(&mut self) {
        if self.playlist_index < self.playlists.len() - 1 {
            self.playlist_index += 1;
            // If moving down would make the selected index go below the visible range,
            // Adjust the offset to keep 5 items visible below the selected index
            if self.playlist_index - self.offset > 5 {
                self.offset += 1;
            }
        }
    }
    pub fn get_playlists(&mut self, list: Vec<SimplifiedPlaylist>) {
        self.playlists = list;
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
            Action::SelectPlaylist(idx) => {}
            Action::MoveUp => self.move_up(),
            Action::MoveDown => self.move_down(),
            _ => {}
        }
        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>>  {
        let action = match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Enter => Action::SelectPlaylist(self.playlist_index),
            KeyCode::Up | KeyCode::Char('k') => Action::MoveUp,
            KeyCode::Down | KeyCode::Char('j') => Action::MoveDown,
            _ => Action::Resume,
        };
        Ok(Some(action))
    }

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

        let playlists: Vec<ListItem> = self.playlists
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

        Ok(())
    }
}
