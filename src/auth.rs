use chrono::offset::Utc;
use chrono::Duration;
use rspotify::{
    model::AlbumId, model::ArtistId, prelude::*, scopes, AuthCodeSpotify, ClientCredsSpotify,
    Config, Credentials, OAuth,
};

struct Spotify {
    spotify: AuthCodeSpotify
}


pub async fn auth() {
    let config = Config {
        ..Default::default()
    };

    let creds = Credentials::from_env().unwrap();
    let oauth = OAuth::from_env(scopes!("user-follow-read user-follow-modify")).unwrap();

    let spotify = AuthCodeSpotify::with_config(creds.clone(), oauth, config.clone());
    let url = spotify.get_authorize_url(false).unwrap();
    spotify
        .prompt_for_token(&url)
        .await
        .expect("couldn't authenticate successfully");
}
