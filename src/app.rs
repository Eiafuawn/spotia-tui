use crate::spotify::Spotify;
use rspotify::model::SimplifiedPlaylist;

#[derive(Debug, Default)]
pub struct App {
    pub spotify: Spotify,
    pub playlists: Vec<SimplifiedPlaylist>,
    pub selected_playlist_index: usize,
    pub offset: usize,
    pub should_quit: bool,
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
    pub fn download_playlist(&self) { }

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
