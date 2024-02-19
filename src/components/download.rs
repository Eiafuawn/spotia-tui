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

    fn reset(&mut self) {
        self.download_output.clear();
        self.mode = Mode::Idle;
    }
}

impl Component for Download {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::SelectPlaylist(_) => self.mode = Mode::Downloading,
            Action::SelectActivePlaylist(_) => self.mode = Mode::Downloading,
            Action::BackHome => self.reset(),
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
        let row_count = rect.rows().count();
        let out_count = self.download_output.lines().count();
        if out_count > row_count {
            let diff = out_count - row_count;
            let lines = self.download_output.lines().skip(diff).collect::<Vec<_>>();
            self.download_output = lines.join("\n");
        }
        match self.mode {
            Mode::Downloading => {
                let output = Paragraph::new(self.download_output.clone())
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .block(Block::default().borders(Borders::ALL).title("Output"));

                f.render_widget(output, rect);
            }
            _ => {}
        }

        Ok(())
    }
}
