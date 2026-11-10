use super::model::Session;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub(super) fn on_key(session: &mut Session, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('q') => true,
        KeyCode::Down => {
            session.move_down();
            false
        }
        KeyCode::Up => {
            session.move_up();
            false
        }
        _ => false,
    }
}
