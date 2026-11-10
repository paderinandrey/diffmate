use super::model::Session;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub(super) enum Command {
    Quit,
    Open,
    Back,
    Down,
    Up,
    Keep,
    Drop,
    KeepRest,
    Edit,
    Include,
    Ignored,
}

pub(super) fn on_key(session: &Session, key: KeyEvent) -> Command {
    match key.code {
        KeyCode::Char('q') => Command::Quit,
        KeyCode::Enter if !session.on_draft() => Command::Open,
        KeyCode::Esc if session.on_draft() => Command::Back,
        KeyCode::Down => Command::Down,
        KeyCode::Up => Command::Up,
        KeyCode::Char('k') if session.on_draft() => Command::Keep,
        KeyCode::Char('d') if session.on_draft() => Command::Drop,
        KeyCode::Char('K') if session.on_draft() => Command::KeepRest,
        KeyCode::Char('e') if session.on_draft() => Command::Edit,
        KeyCode::Char('i') if session.on_draft() => Command::Include,
        _ => Command::Ignored,
    }
}
