use crate::auth::SpotifyAuth;

#[derive(Debug, Default)]
pub struct App {
    pub playlists: Vec<String>,
    pub selected_playlist: String,
    pub should_quit: bool
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
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
