use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, CurrentScreen};

pub fn update(app: &mut App, key_event: KeyEvent) {
        match app.current_screen {
            CurrentScreen::Main => match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => app.quit(),
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        app.quit();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                KeyCode::Enter => app.current_screen = crate::app::CurrentScreen::Editing,
                _ => {}
            }
            CurrentScreen::Editing => match key_event.code {
                KeyCode::Enter => {
                    let _ = app.download_playlist();
                }
                KeyCode::Backspace => {
                    app.key_input.pop();
                }
                KeyCode::Esc => {
                    app.current_screen = CurrentScreen::Main;
                }
                KeyCode::Char(value) => {
                    app.key_input.push(value);
                }
                _ => {}
            }
            _ => {}
        };
}
