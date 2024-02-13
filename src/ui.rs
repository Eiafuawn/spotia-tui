use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::*,
};

use crate::app::{App, CurrentScreen};

/// Renders the user interface widgets.
pub fn render(app: &mut App, f: &mut Frame) {
    let main_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ],
    )
    .split(f.size());
    f.render_widget(
        Block::new()
            .borders(Borders::TOP)
            .title("Select a playlist to download"),
        main_layout[0],
    );
    f.render_widget(
        Block::new().borders(Borders::TOP).title("Status Bar"),
        main_layout[2],
    );

    let chunks = Layout::new(
        Direction::Horizontal,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .split(main_layout[1]);

    let playlists: Vec<ListItem> = app
        .playlists
        .iter()
        .skip(app.offset)
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected_playlist_index - app.offset {
                Style::default().bg(Color::Yellow).fg(Color::Black)
            } else {
                Style::default()
            };
            ListItem::new(item.name.clone()).style(style)
        })
        .collect();

    let menu = List::new(playlists.clone())
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

    f.render_widget(menu, chunks[0]);

    if let CurrentScreen::Editing = app.current_screen {
        f.render_widget(Clear, f.size()); //this clears the entire screen and anything already drawn
        let popup_block = Block::default()
            .title("Choose your folder")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Gray));

        let popup = Paragraph::new(app.key_input.clone())
            .style(Style::default().bg(Color::White).fg(Color::Black))
            .block(popup_block);

        let area = centered_rect(60, 25, f.size());
        f.render_widget(popup, area);
    }}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
