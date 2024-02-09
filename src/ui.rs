use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::*,
};

use crate::app::App;

pub fn render(app: &App, f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let list_items: Vec<ListItem> = app
        .playlists
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected_playlist_index {
                Style::default().bg(Color::Yellow).fg(Color::Black)
            } else {
                Style::default()
            };
            ListItem::new(item.name.clone()).style(style)
        })
        .collect();

    let menu = List::new(list_items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

    f.render_widget(menu, chunks[0]); 
}
/*
pub fn render(app: &mut App, f: &mut Frame) {
    f.render_widget(Paragraph::new(format!(
            "Selected Playlist: {}",
            app.playlists_names.join("\n")
    )).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Playlists downloader")
            .title_alignment(Alignment::Center)
        )
        .style(Style::default().fg(Color::Blue))
        .alignment(Alignment::Center),
        f.size()
        );
    }

// Define an enum to represent the different UI states

fn menu(frame: &mut Frame) {
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
        Direction::Vertical,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .split(main_layout[1]);
    frame.render_widget(
        Paragraph::new("Hello left block")
            .block(Block::default().borders(Borders::ALL).title("Left")),
        inner_layout[0],
    );
    frame.render_widget(
        Paragraph::new("Hello right block")
            .block(Block::default().borders(Borders::ALL).title("Right")),
        inner_layout[1],
    );
}

fn downloads(frame: &mut Frame) {
    // Define the UI for the 'Other' state here
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

fn startup() -> Result<()> {
  enable_raw_mode()?;
  execute!(std::io::stderr(), EnterAlternateScreen)?;
  Ok(())
}

fn shutdown() -> Result<()> {
  execute!(std::io::stderr(), LeaveAlternateScreen)?;
  disable_raw_mode()?;
  Ok(())
}

fn run() -> Result<()> {
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
    let mut app = App { 
        main_menu: menu, 
        downloads_menu: downloads 
    };

    loop {
        t.draw(|f| (app.main_menu)(f))?;
        if handle_events()? {
            break;
        }
    }
    Ok(())
}*/
