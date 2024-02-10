use futures::stream::TryStreamExt;
use futures_util::pin_mut;
use rspotify::{
    model::{playlist::{ FullPlaylist, SimplifiedPlaylist },  Market, Country }, prelude::*, scopes, AuthCodeSpotify,
    Config, Credentials, OAuth 
};

use crate::app::App;


#[derive(Debug, Default)]
pub struct Spotify {
    spotify: AuthCodeSpotify,
}

impl Spotify {
    pub async fn new() -> Self {
        let config = Config { ..Default::default() };

        let creds = Credentials::from_env().unwrap();
        let oauth = OAuth::from_env(scopes!("playlist-read-private playlist-read-collaborative")).unwrap();

        let spotify = AuthCodeSpotify::with_config(creds.clone(), oauth, config.clone());
        let url = spotify.get_authorize_url(false).unwrap();
        spotify.prompt_for_token(&url) .await.unwrap();
        
       Self {
           spotify
       } 
    }

    pub async fn get_playlists(&self) -> Vec<SimplifiedPlaylist> {
        let stream = self.spotify.current_user_playlists();
        let mut playlists = vec![];
        pin_mut!(stream);
        println!("Items (blocking): ");
        while let Some(item) = stream.try_next().await.unwrap() {
            playlists.push(item);
        }
        playlists
    }

    pub fn get_playlist_uri(&self, app: &App) -> String {
        app.playlists[app.selected_playlist_index].id.uri()
    }

    pub async fn get_tracks(&self, app: &App) -> FullPlaylist {
        let id = app.playlists[app.selected_playlist_index].id.clone();
        self.spotify.playlist(
            id,
            None,
            Some(Market::Country(Country::Switzerland))).await.unwrap()
    }
}
