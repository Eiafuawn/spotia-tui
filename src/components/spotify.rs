use futures::TryStreamExt;
use futures_util::pin_mut;
use std::{
    collections::HashMap, 
    time::Duration,
    fs, 
    env,
    io::{self, BufRead, BufReader, Write},
    path::Path, process::{Command, Stdio}
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

#[derive(Default)]
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
    fn select_playlist(&mut self, dir: String, idx: usize) -> Result<()>{
        let url = self.get_playlist_url(idx);
        let name = self.get_playlist_name(idx)
            .replace(' ', "");
        let path = dir.clone() + "/" + &name;
        let dir_path = Path::new(&path);

        if !dir_path.exists() {
            if let Err(err) = fs::create_dir_all(dir_path) {
                self.download_output.push_str(&format!("Error creating directory: {}\n", err));
                            
            } else {
                self.download_output.push_str(&format!("Directory {} created successfully!\n", dir));
                self.download_playlist(url, dir)?;
            }
        } else {
            self.download_output.push_str(&format!("Directory {} already exists!\n", dir));
            self.sync_playlist(dir_path)?;
        }

        Ok(())
    }

    //// Sync the selected playlist
    fn sync_playlist(&mut self, dir: &Path) -> Result<()> {
        self.download_output.push_str("Syncing playlist...\n");
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
                if let Some(tx) = &self.command_tx {
                    tx.send(Action::Downloading).unwrap();
                }
            });

        Ok(())
    }

    //// Download the selected playlist
    fn download_playlist(&mut self, url: String, dir: String) -> Result<()> {
        self.download_output.push_str("Downloading playlist...");
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

        reader.lines().map_while(|line| line.ok()).for_each(|line| {
            self.download_output.push_str(&line);
            self.download_output.push('\n');
        });

        Ok(())
    }
}

impl Component for Spotify {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        #[allow(clippy::single_match)]
        match action {
            Action::SelectPlaylist(dir, idx) =>  self.select_playlist(dir, idx)?,
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let chunks = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .split(area);
        let output = Paragraph::new(self.download_output.clone())
            .style(Style::default().fg(Color::Black).bg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("Output"));

        f.render_widget(output, chunks[1]);

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
