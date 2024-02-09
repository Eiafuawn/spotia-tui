use std::sync::mpsc;
use crate::app::App;
use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::*,
    text::Text,
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
    //render_download_output(f, chunks[1], app);
}

fn render_download_output(f: &mut Frame, chunk: Rect, app: &App) {
    let (sender, receiver) = mpsc::channel();

    // Start downloading the playlist in a separate thread
    app.download_playlist(sender.clone());

    // Render the output in the second chunk (chunks[1]) dynamically
    let mut output_content = Vec::new();
    while let Ok(line) = receiver.recv() {
        output_content.push(Text::raw(line));
        // Re-render the output block with updated content
        let output_block = Block::default()
            .title("Output")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        for text in output_content.iter() {
            f.render_widget(text.clone(), chunk);
        }
        f.render_widget(output_block, chunk);
    }
}
