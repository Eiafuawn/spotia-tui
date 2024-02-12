use crate::spotify::Spotify;
use rspotify::model::SimplifiedPlaylist;
use std::{
    fs, 
    io::{self, BufRead, BufReader, Write}, 
    path::Path, 
    process::{Command, Stdio},
};

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
    pub fn select_playlist(&mut self) -> io::Result<()>{
        let url = self.spotify.get_playlist_url(self);
        let name = self.spotify.get_playlist_name(self).replace(' ', "");
        let dir = self.key_input.clone() + "/" + &name;
        let dir_path = Path::new(&dir);

        if let Err(err) = clear_terminal() {
            eprintln!("Error clearing terminal: {}", err);
        }

        if !dir_path.exists() {
            if let Err(err) = fs::create_dir_all(dir_path) {
                eprintln!("Error creating directory: {}", err);
            } else {
                println!("Directory {} created successfully!", dir);
                self.download_playlist(url, dir)?;
            }
        } else {
            println!("Directory already exists!");
            self.sync_playlist(dir_path)?;
        }
        Ok(())
    }

    //// Sync the selected playlist
    fn sync_playlist(&mut self, dir: &Path) -> io::Result<()> {
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
            .for_each(|line| { println!("{}", line); });

        self.key_input.clear();
        self.downloaded = false;
        
        Ok(())
    }
    
    fn download_playlist(&mut self, url: String, dir: String) -> io::Result<()> {
        println!("Downloading playlist...");
        let stdout = Command::new("spotdl")
            .args(["sync".to_string(), url, "--save-file".to_string(), "save.spotdl".to_string(), "--simple-tui".to_string()])
            .current_dir(dir)
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

fn clear_terminal() -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    // Print ANSI escape code for clearing the terminal screen
    handle.write_all(b"\x1B[2J\x1B[1;1H")?;

    Ok(())
}
