use super::model::Session;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub(super) enum Command {
    Quit,
    Open,
    Back,
    Down,
    Up,
    Ignored,
}

pub(super) fn on_key(session: &Session, key: KeyEvent) -> Command {
    match key.code {
        KeyCode::Char('q') => Command::Quit,
        KeyCode::Enter if !session.on_draft() => Command::Open,
        KeyCode::Esc if session.on_draft() => Command::Back,
        KeyCode::Down => Command::Down,
        KeyCode::Up => Command::Up,
        _ => Command::Ignored,
    }
}
