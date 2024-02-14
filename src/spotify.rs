use futures::stream::TryStreamExt;
use futures_util::pin_mut;
use rspotify::{
    model::{
        playlist::{FullPlaylist, PlaylistItem, SimplifiedPlaylist}, Country, FullTrack, Market, PlayableItem
    },
    prelude::*,
    scopes, AuthCodeSpotify, Config, Credentials, OAuth,
};

use crate::app::App;

#[derive(Debug, Default)]
pub struct Spotify {
    spotify: AuthCodeSpotify,
    pub playlists: Vec<SimplifiedPlaylist>,
}

impl Spotify {
    pub async fn new() -> Self {
        let config = Config {
            ..Default::default()
        };

        let creds = Credentials::from_env().unwrap();
        let oauth =
            OAuth::from_env(
                scopes!("playlist-read-private playlist-read-collaborative")
            ).unwrap();

        let spotify = AuthCodeSpotify::with_config(creds.clone(), oauth, config.clone());
        let url = spotify.get_authorize_url(false).unwrap();
        spotify.prompt_for_token(&url).await.unwrap();
        let playlists = get_playlists(spotify.clone()).await;

        Self { spotify, playlists }
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
}

async fn get_playlists(spotify: AuthCodeSpotify) -> Vec<SimplifiedPlaylist> {
    let stream = spotify.current_user_playlists();
    let mut playlists = vec![];
    pin_mut!(stream);
    while let Some(item) = stream.try_next().await.unwrap() {
        playlists.push(item);
    }
    playlists
}
