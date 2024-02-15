use std::time::Instant;

use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};

use super::Component;
use crate::{action::Action, mode::Mode, tui::Frame};

#[derive(Default)]
pub struct Download {
    mode: Mode,
    download_output: String,
}

impl Download {
    pub fn new() -> Self {
        Self {
            mode: Mode::Idle,
            download_output: String::new(),
        }
    }
}

impl Component for Download {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::EnterEditing => self.mode = Mode::SelectingDir,
            Action::SelectPlaylist(_, _) => self.mode = Mode::Downloading,
            Action::Downloading(output) => {
                self.download_output.push_str(&output);
                self.download_output.push('\n');
            }
            _ => {}
        }
        Ok(None)
    }

    #[allow(clippy::single_match)]
    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
        let chunks = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .split(rect);
        match self.mode {
            Mode::Downloading => {
                let output = Paragraph::new(self.download_output.clone())
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .block(Block::default().borders(Borders::ALL).title("Output"));

                f.render_widget(output, chunks[1]);
            }
            _ => {},
        }

        Ok(())
    }
}
