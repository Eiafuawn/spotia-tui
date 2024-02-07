use std::io::{self, stdout};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};
use chrono::offset::Utc;
use chrono::Duration;
use rspotify::{
    model::AlbumId, model::ArtistId, prelude::*, scopes, AuthCodeSpotify, ClientCredsSpotify,
    Config, Credentials, OAuth,
};


fn ui(frame: &mut Frame) {
    let main_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ],
    )
    .split(frame.size());
    frame.render_widget(
        Block::new().borders(Borders::TOP).title("Title Bar"),
        main_layout[0],
    );
    frame.render_widget(
        Block::new().borders(Borders::TOP).title("Status Bar"),
        main_layout[2],
    );

    let inner_layout = Layout::new(
        Direction::Horizontal,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .split(main_layout[1]);
    frame.render_widget(
        Paragraph::new("Hello left block").block(
        Block::default().borders(Borders::ALL).title("Left")),
        inner_layout[0],
    );
    frame.render_widget(
        Paragraph::new("Hello right block").block(
        Block::default().borders(Borders::ALL).title("Right")),
        inner_layout[1],
    );
}

fn tui() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(ui)?;
        should_quit = handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn handle_events() -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

async fn expire_token<S: BaseClient>(spotify: &S) {
    let token_mutex = spotify.get_token();
    let mut token = token_mutex.lock().await.unwrap();
    let token = token.as_mut().expect("Token can't be empty as this point");
    // In a regular case, the token would expire with time. Here we just do
    // it manually.
    let now = Utc::now().checked_sub_signed(Duration::seconds(10));
    token.expires_at = now;
    token.expires_in = Duration::seconds(0);
    // We also use a garbage access token to make sure it's actually
    // refreshed.
    token.access_token = "garbage".to_owned();
}

async fn with_auth(creds: Credentials, oauth: OAuth, config: Config) {
    // In the first session of the application we authenticate and obtain the
    // refresh token.
    println!(">>> Session one, obtaining refresh token and running some requests:");
    let spotify = AuthCodeSpotify::with_config(creds.clone(), oauth, config.clone());
    let url = spotify.get_authorize_url(false).unwrap();
    // This function requires the `cli` feature enabled.
    spotify
        .prompt_for_token(&url)
        .await
        .expect("couldn't authenticate successfully");

    // We can now perform requests
    //auth_code_do_things(&spotify).await;
    println!(">>> Session one, done");

    // Manually expiring the token.
    expire_token(&spotify).await;

    // Without automatically refreshing tokens, this would cause an
    // authentication error when making a request, because the auth token is
    // invalid. However, since it will be refreshed automatically, this will
    // work.
    println!(">>> Session two, the token should expire, then re-auth automatically");
    //auth_code_do_things(&spotify).await;
    println!(">>> Session two, done");
}

#[tokio::main]
async fn main() {

    // Enabling automatic token refreshing in the config
    let config = Config {
        ..Default::default()
    };

    // May require the `env-file` feature enabled if the environment variables
    // aren't configured manually.
    let creds = Credentials::from_env().unwrap();
    let oauth = OAuth::from_env(scopes!("user-follow-read user-follow-modify")).unwrap();

    with_auth(creds.clone(), oauth, config.clone()).await;
    let _ = tui();
}
