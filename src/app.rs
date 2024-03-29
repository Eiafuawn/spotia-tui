use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use tokio::sync::mpsc;

use crate::{
    action::Action,
    components::{
        download::Download, fps::FpsCounter, home::Home, manager::Manager, spotify::Spotify,
        Component,
    },
    config::Config,
    mode::Mode,
    tui,
};

pub struct App {
    pub config: Config,
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub components: Vec<Box<dyn Component>>,
    pub displays: Vec<Box<dyn Component>>,
    pub should_quit: bool,
    pub should_suspend: bool,
    pub mode: Mode,
    pub last_tick_key_events: Vec<KeyEvent>,
}

impl App {
    pub fn new(tick_rate: f64, frame_rate: f64, spotify: Spotify) -> Result<Self> {
        let home = Home::new(spotify.playlists.clone());
        let manager = Manager::new();
        let fps = FpsCounter::default();
        let config = Config::new()?;
        let download = Download::new();
        let mode = Mode::Input;
        Ok(Self {
            tick_rate,
            frame_rate,
            components: vec![Box::new(home), Box::new(fps), Box::new(download)],
            displays: vec![Box::new(manager), Box::new(spotify)],
            should_quit: false,
            should_suspend: false,
            config,
            mode,
            last_tick_key_events: Vec::new(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();

        let mut tui = tui::Tui::new()?
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        // tui.mouse(true);
        tui.enter()?;

        for component in self.components.iter_mut() {
            component.register_action_handler(action_tx.clone())?;
        }

        for component in self.components.iter_mut() {
            component.register_config_handler(self.config.clone())?;
        }

        for component in self.components.iter_mut() {
            component.init(tui.size()?)?;
        }

        for display in self.displays.iter_mut() {
            display.register_action_handler(action_tx.clone())?;
        }

        for display in self.displays.iter_mut() {
            display.register_config_handler(self.config.clone())?;
        }

        for display in self.displays.iter_mut() {
            display.init(tui.size()?)?;
        }

        loop {
            if let Some(e) = tui.next().await {
                match e {
                    tui::Event::Quit => action_tx.send(Action::Quit)?,
                    tui::Event::Tick => action_tx.send(Action::Tick)?,
                    tui::Event::Render => action_tx.send(Action::Render)?,
                    tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
                    tui::Event::Key(key) => {
                        if let Some(keymap) = self.config.keybindings.get(&self.mode) {
                            if let Some(action) = keymap.get(&vec![key]) {
                                log::info!("Got action: {action:?}");
                                action_tx.send(action.clone())?;
                            } else {
                                // If the key was not handled as a single key action,
                                // then consider it for multi-key combinations.
                                self.last_tick_key_events.push(key);

                                // Check for multi-key combinations
                                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                                    log::info!("Got action: {action:?}");
                                    action_tx.send(action.clone())?;
                                }
                            }
                        };
                    }
                    _ => {}
                }
                for component in self.components.iter_mut() {
                    if let Some(action) = component.handle_events(Some(e.clone()))? {
                        action_tx.send(action)?;
                    }
                }
                match self.mode {
                    Mode::Manager => {
                        if let Some(action) = self.displays[0].handle_events(Some(e.clone()))? {
                            action_tx.send(action)?;
                        }
                    }
                    Mode::Downloader => {
                        if let Some(action) = self.displays[1].handle_events(Some(e.clone()))? {
                            action_tx.send(action)?;
                        }
                    }
                    _ => {}
                }
            }

            while let Ok(action) = action_rx.try_recv() {
                if action != Action::Tick && action != Action::Render {
                    log::debug!("{action:?}");
                }
                match action {
                    Action::Tick => {
                        self.last_tick_key_events.drain(..);
                    }
                    Action::SelectFolder(_) => self.mode = Mode::Home,
                    Action::EnterEditing => self.mode = Mode::Input,
                    Action::EnterDownloader => self.mode = Mode::Downloader,
                    Action::EnterManager => self.mode = Mode::Manager,
                    Action::DownloadFinished => self.mode = Mode::Waiting,
                    Action::BackHome => self.mode = Mode::Home,
                    Action::Quit => self.should_quit = true,
                    Action::Suspend => self.should_suspend = true,
                    Action::Resume => self.should_suspend = false,
                    Action::Resize(w, h) => {
                        tui.resize(Rect::new(0, 0, w, h))?;
                        tui.draw(|f| {
                            f.render_widget(
                                Block::new()
                                    .borders(Borders::TOP)
                                    .title("Select a playlist to download"),
                                main_layout(f.size())[0],
                            );
                            for component in self.components.iter_mut() {
                                let r = component.draw(f, main_layout(f.size())[1]);
                                if let Err(e) = r {
                                    action_tx
                                        .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                        .unwrap();
                                }
                            }
                            let r = match self.mode {
                                Mode::Manager => self.displays[0].draw(f, main_layout(f.size())[1]),
                                Mode::Downloader => {
                                    self.displays[1].draw(f, main_layout(f.size())[1])
                                }
                                _ => Ok(()),
                            };
                            if let Err(e) = r {
                                action_tx
                                    .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                    .unwrap();
                            }
                        })?;
                    }
                    Action::Render => {
                        tui.draw(|f| {
                            f.render_widget(
                                Block::new()
                                    .borders(Borders::TOP)
                                    .title("Select a playlist to download"),
                                main_layout(f.size())[0],
                            );
                            for component in self.components.iter_mut() {
                                let r = component.draw(f, main_layout(f.size())[1]);
                                if let Err(e) = r {
                                    action_tx
                                        .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                        .unwrap();
                                }
                            }
                            for display in self.displays.iter_mut() {
                                let r = display.draw(f, main_layout(f.size())[1]);
                                if let Err(e) = r {
                                    action_tx
                                        .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                        .unwrap();
                                }
                            }
                        })?;
                    }
                    _ => {}
                }
                for component in self.components.iter_mut() {
                    if let Some(action) = component.update(action.clone())? {
                        action_tx.send(action)?
                    };
                }
                for display in self.displays.iter_mut() {
                    if let Some(action) = display.update(action.clone())? {
                        action_tx.send(action)?
                    };
                }
            }
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                tui = tui::Tui::new()?
                    .tick_rate(self.tick_rate)
                    .frame_rate(self.frame_rate);
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }
}

fn main_layout(size: Rect) -> Rc<[Rect]> {
    Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ],
    )
    .split(size)
}
