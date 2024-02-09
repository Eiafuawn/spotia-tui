use rspotify::model::SimplifiedPlaylist;

use crate::spotify::playlists;

pub enum Action {
    Tick,
    MoveUp,
    MoveDown,
    Quit,
    None,
}


#[derive(Debug, Default)]
pub struct App {
    pub playlists: Vec<SimplifiedPlaylist>,
    pub selected_playlist_index: usize,
    pub should_quit: bool
}

impl App {
    /// Constructs a new instance of [`App`].
    pub async fn new() -> Self {
        let playlists = playlists().await;                                        
        Self {
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
    
    // Download the selected playlist
    pub fn download(&mut self) {}
    
    // Search for a playlist
    pub fn search(&mut self) {}

    pub fn move_up(&mut self) {
        if self.selected_playlist_index > 0 {
            self.selected_playlist_index -= 1;
        }
    }
    pub fn move_down(&mut self) {
        if self.selected_playlist_index < self.playlists.len() - 1 {
            self.selected_playlist_index += 1;
        }
    }
}
