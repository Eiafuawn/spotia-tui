use rspotify::model::SimplifiedPlaylist;

use crate::spotify::playlists;

#[derive(Debug, Default)]
pub struct App {
    pub playlists: Vec<SimplifiedPlaylist>,
    pub playlists_names: Vec<String>,
    pub selected_playlist: String,
    pub should_quit: bool
}

impl App {
    /// Constructs a new instance of [`App`].
    pub async fn new() -> Self {
        let playlists = playlists().await;
        let playlists_names: Vec<String> = playlists.iter().map(|p| p.name.clone()).collect();
        Self {
            playlists,
            playlists_names,
            ..Default::default()
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set should_quit to true to quit the application.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
    
    // Download the selected playlist
    pub fn download(&mut self) {}
    
    // Search for a playlist
    pub fn search(&mut self) {}

    pub fn move_up(&mut self) {}
    pub fn move_down(&mut self) {}
}
