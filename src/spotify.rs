use futures::stream::TryStreamExt;
use futures_util::pin_mut;
use rspotify::{
    model::playlist::SimplifiedPlaylist, prelude::*, scopes, AuthCodeSpotify,
    Config, Credentials, OAuth 
};

async fn auth() -> AuthCodeSpotify {
    let config = Config { ..Default::default() };

    let creds = Credentials::from_env().unwrap();
    let oauth = OAuth::from_env(scopes!("playlist-read-private playlist-read-collaborative")).unwrap();

    let spotify = AuthCodeSpotify::with_config(creds.clone(), oauth, config.clone());
    let url = spotify.get_authorize_url(false).unwrap();
    spotify.prompt_for_token(&url) .await.unwrap();
    
    spotify
}

async fn get_playlists(spotify: &AuthCodeSpotify) -> Vec<SimplifiedPlaylist> {
    let stream = spotify.current_user_playlists();
    let mut playlists = vec![];
    pin_mut!(stream);
    println!("Items (blocking): ");
    while let Some(item) = stream.try_next().await.unwrap() {
        playlists.push(item);
    }
    playlists
}

pub async fn playlists() -> Vec<SimplifiedPlaylist> {
    let spotify = auth().await;
    get_playlists(&spotify).await
}
