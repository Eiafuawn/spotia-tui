use crossterm::style::Stylize;
use futures::TryStreamExt;
use futures_util::pin_mut;
use std::{
    collections::HashMap,
    env, fs,
    io::{self, BufRead, BufReader, Write},
    path::Path,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::Duration,
};

use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};
use rspotify::{
    model::{
        playlist::{FullPlaylist, PlaylistItem, SimplifiedPlaylist},
        Country, FullTrack, Market, PlayableItem,
    },
    prelude::*,
    scopes, AuthCodeSpotify, Config, Credentials, OAuth,
};

use super::Component;
use crate::{action::Action, tui::Frame};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default, Clone)]
pub struct Spotify {
    spotify: AuthCodeSpotify,
    pub playlists: Vec<SimplifiedPlaylist>,
    command_tx: Option<UnboundedSender<Action>>,
    download_output: String,
    config: Config,
}

impl Spotify {
    pub async fn new() -> Self {
        let config = Config {
            ..Default::default()
        };

        let creds = Credentials::from_env().unwrap();
        let oauth =
            OAuth::from_env(scopes!("playlist-read-private playlist-read-collaborative")).unwrap();

        let spotify = AuthCodeSpotify::with_config(creds.clone(), oauth, config.clone());
        let url = spotify.get_authorize_url(false).unwrap();
        spotify.prompt_for_token(&url).await.unwrap();
        let playlists = get_playlists(&spotify).await;

        Self {
            spotify,
            playlists,
            ..Default::default()
        }
    }

    pub fn get_playlist_url(&self, idx: usize) -> String {
        self.playlists[idx].id.url()
    }

    pub fn get_playlist_name(&self, idx: usize) -> String {
        self.playlists[idx].name.clone()
    }

    pub async fn get_tracks(&self, idx: usize) -> Vec<FullTrack> {
        let playlist = self.playlists[idx].id.clone();
        let stream = self.spotify.playlist_items(playlist, None, None);
        pin_mut!(stream);
        let mut tracks = vec![];
        while let Some(item) = stream.try_next().await.unwrap() {
            let track = item.track;
            if let Some(PlayableItem::Track(track)) = track {
                tracks.push(track);
            }
        }
        tracks
    }

    //// Get the playlists and launches the download/sync
    fn select_playlist(&mut self, dir: String, idx: usize) -> Result<()> {
        let url = self.get_playlist_url(idx);
        let name = self.get_playlist_name(idx).replace(' ', "");
        let path = dir.clone() + "/" + &name;
        let dir_path = Path::new(&path);

        let download_output = Arc::new(Mutex::new(self.download_output.clone())); // Use Arc<Mutex<_>> to share across threads
        let mut spotify_clone = self.clone();
        let url_clone = url.clone();
        let dir_path_clone = dir_path.to_owned();
        let cmd_tx = self.command_tx.clone();

        if !dir_path.exists() {
            if let Err(err) = fs::create_dir_all(dir_path) {
                download_output
                    .lock()
                    .unwrap()
                    .push_str(&format!("Error creating directory: {}\n", err));
            } else {
                download_output
                    .lock()
                    .unwrap()
                    .push_str(&format!("Directory {} created successfully!\n", &dir));

                // Spawn a blocking task to run download_playlist
                tokio::spawn(async move {
                    if let Err(err) = spotify_clone
                        .download_playlist(url_clone, &dir_path_clone)
                        .await
                    {
                        download_output
                            .lock()
                            .unwrap()
                            .push_str(&format!("Error downloading playlist: {}\n", err));
                    }
                });
            }
        } else {
            tokio::spawn(async move {
                if let Err(err) = spotify_clone.sync_playlist(&dir_path_clone).await {
                    download_output
                        .lock()
                        .unwrap()
                        .push_str(&format!("Error downloading playlist: {}\n", err));
                }
            });
        }

        Ok(())
    }

    //// Sync the selected playlist
    async fn sync_playlist(&mut self, dir: &Path) -> Result<()> {
        self.send_output("Syncing playlist...".to_string());
        let stdout = Command::new("spotdl")
            .args(["sync".to_string(), "save.spotdl".to_string()])
            .current_dir(dir)
            .stdout(Stdio::piped()) // Redirect stdout to a pipe
            .spawn()?
            .stdout
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to execute command"))?;

        let reader = BufReader::new(stdout);

        reader
            .lines()
            .map_while(|line| line.ok())
            .for_each(|line| self.send_output(line));

        self.send_output("Syncing finished!".to_string());

        if let Some(tx) = &self.command_tx {
            tx.send(Action::DownloadFinished).unwrap();
        }

        Ok(())
    }

    //// Download the selected playlist
    async fn download_playlist(&mut self, url: String, dir: &Path) -> Result<()> {
        self.send_output("Download started...".to_string());
        let stdout = Command::new("spotdl")
            .args([
                "sync".to_string(),
                url,
                "--save-file".to_string(),
                "save.spotdl".to_string(),
                "--simple-tui".to_string(),
            ])
            .current_dir(dir)
            .stdout(Stdio::piped()) // Redirect stdout to a pipe
            .spawn()?
            .stdout
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to execute command"))?;

        let reader = BufReader::new(stdout);

        reader
            .lines()
            .map_while(|line| line.ok())
            .for_each(|line| self.send_output(line));
        self.send_output("Download finished!".to_string());

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

impl Component for Spotify {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        #[allow(clippy::single_match)]
        match action {
            Action::SelectPlaylist(dir, idx) => self.select_playlist(dir, idx)?,
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        Ok(())
    }
}

async fn get_playlists(spotify: &AuthCodeSpotify) -> Vec<SimplifiedPlaylist> {
    let stream = spotify.current_user_playlists();
    let mut playlists = vec![];
    pin_mut!(stream);
    while let Some(item) = stream.try_next().await.unwrap() {
        playlists.push(item);
    }
    playlists
}
