use crate::app::App;
use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::*,
};


pub fn render(app: &App, f: &mut Frame) {
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
        Block::new().borders(Borders::TOP).title("Select a playlist to download"),
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
