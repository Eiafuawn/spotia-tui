use std::time::Instant;

use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};

use super::Component;
use crate::{action::Action, tui::Frame};

#[derive(Default)]
pub struct Download {
    download_output: String,
}

impl Download {
    pub fn new() -> Self {
        Self {
            download_output: String::new(),
        }
    }
}

impl Component for Download {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        #[allow(clippy::single_match)]
        match action {
            Action::Downloading(output) => {
            self.download_output.push_str(&output);
            self.download_output.push('\n');
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
        let chunks = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .split(rect);
        let output = Paragraph::new(self.download_output.clone())
            .style(Style::default().fg(Color::Black).bg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("Output"));

        f.render_widget(output, chunks[1]);

        Ok(())
    }
}
