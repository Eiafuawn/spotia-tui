use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};

use super::Component;
use crate::{action::Action, tui::Frame};

#[derive(Debug, Clone, Default)]
pub struct Manager {
    active_playlist: PathBuf,
    playlists: Vec<PathBuf>,
}

impl Manager {
    pub fn new() -> Self {
        Manager::default()
    }

    fn scan_dir(&mut self, selected_dir: String) {
        let dirs = fs::read_dir(selected_dir).expect("Failed to read dir");

        for dir in dirs {
            let dir = dir.expect("Failed to get entry");
            let path = dir.path();

            if path.is_dir() {
                self.active_playlist = path;
            } else if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    match ext_str.to_lowercase().as_str() {
                        "zip" | "rar" | "tar" | "7z" => {
                            self.playlists.push(path);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

impl Component for Manager {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        #[allow(clippy::single_match)]
        match action {
            Action::SelectFolder(dir) => self.scan_dir(dir),
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
        Ok(())
    }
}

fn get_dir_name(path: &Path) -> Option<&str> {
    path.file_name()
        .and_then(|name| name.to_str())
}
