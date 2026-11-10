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
    Discard,
    OpenPr,
    Rerun,
    Unpublish,
    Approve,
    Request,
    Comment,
    Accept,
    Cancel,
    Safer,
    Riskier,
    Ignored,
}

pub(super) fn on_key(session: &Session, key: KeyEvent) -> Command {
    if session.confirming() {
        return match key.code {
            KeyCode::Char('q') => Command::Quit,
            KeyCode::Esc => Command::Cancel,
            KeyCode::Enter => Command::Accept,
            KeyCode::Left | KeyCode::Up => Command::Safer,
            KeyCode::Right | KeyCode::Down => Command::Riskier,
            _ => Command::Ignored,
        };
    }
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
        KeyCode::Char('x') => Command::Discard,
        KeyCode::Char('o') => Command::OpenPr,
        KeyCode::Char('n') => Command::Rerun,
        KeyCode::Char('u') => Command::Unpublish,
        KeyCode::Char('a') if session.on_draft() => Command::Approve,
        KeyCode::Char('r') if session.on_draft() => Command::Request,
        KeyCode::Char('c') if session.on_draft() => Command::Comment,
        _ => Command::Ignored,
    }
}
