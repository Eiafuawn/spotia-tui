use std::{
    fs::{self, DirBuilder, File},
    io::{BufRead, BufReader, Error, ErrorKind, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Instant,
};
use zip::write::{FileOptions, ZipWriter};

use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, tui::Frame};

#[derive(Debug, Clone, Default)]
pub struct Manager {
    playlists: Vec<PathBuf>,
    command_tx: Option<UnboundedSender<Action>>,
    dir: String,
}

impl Manager {
    pub fn new() -> Self {
        Manager::default()
    }

    fn scan_dir(&mut self) {
        let dirs = fs::read_dir(self.dir.clone()).expect("Failed to read dir");
        let mut send_dirs = vec![];

        for dir in dirs {
            let dir = dir.expect("Failed to get entry");
            let path = dir.path();

            if path.is_dir() {
                self.playlists.push(path.clone());
                send_dirs.push(get_dir_name(path.as_path()).unwrap().to_string());
            } else if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    match ext_str.to_lowercase().as_str() {
                        "zip" | "rar" | "tar" | "7z" => {
                            self.playlists.push(path.clone());
                            send_dirs.push(get_dir_name(path.as_path()).unwrap().to_string());
                        }
                        _ => {}
                    }
                }
            }
        }
        if let Some(tx) = &self.command_tx {
            tx.send(Action::GetDirs(send_dirs)).unwrap();
        }
    }

    fn select_playlist(&mut self, idx: usize) {
        let mut clone = self.clone();
        let playlist = self.playlists[idx].clone();
        if playlist.is_dir() {
            tokio::spawn(async move {
                if let Err(err) = clone.zip_playlist(idx).await {
                    clone.send_output(format!("Failed to zip playlist: {}", err));
                }
            });
        } else {
            tokio::spawn(async move {
                if let Err(err) = clone.unzip_playlist(idx).await {
                    clone.send_output(format!("Failed to unzip playlist: {}", err));
                }
            });
        }
    }

    async fn zip_playlist(&mut self, idx: usize) -> Result<()> {
        self.send_output("Zipping playlist".to_string());
        let path = self.playlists[idx].clone();
        let name = get_dir_name(&path).unwrap();
        let archive = format!("{}.zip", name);

        let stdout = Command::new("zip")
            .current_dir(self.dir.clone())
            .args(["-rm".to_string(), archive, name.to_string()])
            .stdout(Stdio::piped())
            .spawn()?
            .stdout
            .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to execute command"))?;

        let reader = BufReader::new(stdout);

        reader
            .lines()
            .map_while(|line| line.ok())
            .for_each(|line| self.send_output(line));

        self.send_output("Zipping finished! Press Enter to go back to the menu".to_string());

        if let Some(tx) = &self.command_tx {
            tx.send(Action::DownloadFinished).unwrap();
        }

        Ok(())
    }

    async fn unzip_playlist(&mut self, idx: usize) -> Result<()> {
        self.send_output("Unzipping playlist".to_string());
        let path = self.playlists[idx].clone();
        let name = get_dir_name(&path).unwrap();
        let archive = format!("{}.zip", name);

        let stdout = Command::new("unzip")
            .current_dir(self.dir.clone())
            .arg(name)
            .stdout(Stdio::piped())
            .spawn()?
            .stdout
            .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to execute command"))?;

        let reader = BufReader::new(stdout);

        reader
            .lines()
            .map_while(|line| line.ok())
            .for_each(|line| self.send_output(line));

        Command::new("rm")
            .current_dir(self.dir.clone())
            .arg(archive)
            .spawn()?;

        self.send_output("Unzipping finished! Press Enter to go back to the menu".to_string());

        if let Some(tx) = &self.command_tx {
            tx.send(Action::DownloadFinished).unwrap();
        }

        Ok(())
    }

    fn send_output(&mut self, out: String) {
        if let Some(tx) = &self.command_tx {
            tx.send(Action::Downloading(out)).unwrap();
        }
    }
}

impl Component for Manager {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        #[allow(clippy::single_match)]
        match action {
            Action::SelectFolder(dir) => self.dir = dir,
            Action::EnterManager => self.scan_dir(),
            Action::SelectActivePlaylist(idx) => self.select_playlist(idx),
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
        Ok(())
    }
}

fn get_dir_name(path: &Path) -> Option<&str> {
    path.file_name().and_then(|name| name.to_str())
}
