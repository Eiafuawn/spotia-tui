use crate::app::{App, AppResult, CurrentScreen};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
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
                    app.current_screen = CurrentScreen::Main;
                    app.downloaded = true;
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
    Ok(())
}
