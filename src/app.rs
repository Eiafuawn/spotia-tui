use crate::spotify::Spotify;
use rspotify::model::SimplifiedPlaylist;
use std::{io::{self, BufRead, BufReader}, process::{Command, Stdio}};

#[derive(Debug)]
pub enum CurrentScreen {
    Main,
    Editing, Exiting,
}

impl Default for CurrentScreen {
    fn default() -> Self {
        Self::Main
    }
}

#[derive(Debug, Default)]
pub struct App {
    pub current_screen: CurrentScreen,
    pub spotify: Spotify,
    pub playlists: Vec<SimplifiedPlaylist>,
    pub selected_playlist_index: usize,
    pub offset: usize,
    pub should_quit: bool,
    pub downloaded: bool,
    pub key_input: String,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub async fn new() -> Self {
        let spotify = Spotify::new().await;
        let playlists = spotify.get_playlists().await;
        Self {
            spotify,
            playlists,
            ..Default::default()
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set should_quit to true to quit the application.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    //// Download the selected playlist
    pub fn download_playlist(&mut self) -> io::Result<()>{
        let url = self.spotify.get_playlist_url(self);
        let stdout = Command::new("spotdl")
            .arg(url)
            .current_dir(self.key_input.clone())
            .stdout(Stdio::piped())  // Redirect stdout to a pipe
            .spawn()?
            .stdout
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to execute command"))?;

        let reader = BufReader::new(stdout);

        reader.lines()
            .map_while(|line| line.ok())
            .for_each(|line| { println!("{}", line); });

        self.key_input.clear();
        self.downloaded = false;
        
        Ok(())
    }

    //// Search for a playlist
    pub fn search(&mut self) {}

    /// Moves the selection cursor up.
    pub fn move_up(&mut self) {
        if self.selected_playlist_index > self.offset {
            self.selected_playlist_index -= 1;
            if self.selected_playlist_index > 5 {
                self.offset -= 1;
            }
        }
    }

    //// Moves the selection cursor down.
    pub fn move_down(&mut self) {
        if self.selected_playlist_index < self.playlists.len() - 1 {
            self.selected_playlist_index += 1;
            if self.selected_playlist_index > 5 {
                self.offset += 1;
            }
        }
    }
}
