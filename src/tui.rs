mod draw;
mod edit;
mod keys;
mod model;

#[cfg(test)]
mod tests;

use crate::app::AppConfig;
use crate::error::Error;
use crate::render::sanitize;
use crate::store::Store;
use model::Session;
use ratatui::crossterm::event::{Event, KeyEvent, KeyEventKind};
#[cfg(test)]
use ratatui::{Terminal, backend::TestBackend};
use std::io::{self, IsTerminal as _, stdout};
use std::process::ExitCode;

pub(crate) async fn run(config: &AppConfig) -> ExitCode {
    if !stdout().is_terminal() {
        eprintln!("error: diffmate tui needs a terminal");
        return ExitCode::FAILURE;
    }
    match show(config).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {}", sanitize(&error.to_string()));
            ExitCode::FAILURE
        }
    }
}

async fn show(config: &AppConfig) -> Result<(), Error> {
    let store = Store::open(&config.paths).await?;
    let mut terminal = ratatui::try_init().map_err(Error::io("open the terminal"))?;
    let _restore = Restore;
    let mut session = Session::load(&store).await?;
    let editor = config
        .env
        .get("EDITOR")
        .and_then(|value| value.to_str())
        .map(str::to_owned);
    session.prepare(editor, config.paths.cache_dir().to_path_buf(), true);
    loop {
        terminal
            .draw(|frame| draw::screen(frame, &session))
            .map_err(Error::io("draw the terminal"))?;
        if apply(&mut session, &store, read_key().await?).await? {
            return Ok(());
        }
    }
}

async fn apply(session: &mut Session, store: &Store, key: KeyEvent) -> Result<bool, Error> {
    session.clear_notice();
    match keys::on_key(session, key) {
        keys::Command::Quit => Ok(true),
        keys::Command::Open => {
            session.open_draft(store).await?;
            Ok(false)
        }
        keys::Command::Back => {
            session.close_draft();
            Ok(false)
        }
        keys::Command::Down => {
            session.move_down();
            Ok(false)
        }
        keys::Command::Up => {
            session.move_up();
            Ok(false)
        }
        keys::Command::Keep => {
            session.decide(store, "kept").await?;
            Ok(false)
        }
        keys::Command::Drop => {
            session.decide(store, "dropped").await?;
            Ok(false)
        }
        keys::Command::KeepRest => {
            session.keep_rest(store).await?;
            Ok(false)
        }
        keys::Command::Edit => {
            session.edit_selected(store).await?;
            Ok(false)
        }
        keys::Command::Include => {
            session.include_limitation(store).await?;
            Ok(false)
        }
        keys::Command::Ignored => Ok(false),
    }
}

struct Restore;

impl Drop for Restore {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

async fn read_key() -> Result<KeyEvent, Error> {
    tokio::task::spawn_blocking(|| -> io::Result<KeyEvent> {
        loop {
            if let Event::Key(key) = ratatui::crossterm::event::read()?
                && key.kind == KeyEventKind::Press
            {
                return Ok(key);
            }
        }
    })
    .await?
    .map_err(Error::io("read a key"))
}

#[cfg(test)]
pub(crate) async fn run_scripted(
    store: &Store,
    keys: &[KeyEvent],
    backend: TestBackend,
) -> Result<TestBackend, Error> {
    run_scripted_with(store, keys, backend, None, None).await
}

#[cfg(test)]
pub(crate) async fn run_scripted_with(
    store: &Store,
    keys: &[KeyEvent],
    backend: TestBackend,
    editor: Option<&str>,
    cache: Option<&std::path::Path>,
) -> Result<TestBackend, Error> {
    let mut terminal = Terminal::new(backend).unwrap_or_else(|error| match error {});
    let mut session = Session::load(store).await?;
    if let Some(dir) = cache {
        session.prepare(editor.map(str::to_owned), dir.to_path_buf(), false);
    }
    draw_scripted(&mut terminal, &session);
    for key in keys {
        if apply(&mut session, store, *key).await? {
            break;
        }
        draw_scripted(&mut terminal, &session);
    }
    Ok(terminal.backend().clone())
}

#[cfg(test)]
fn draw_scripted(terminal: &mut Terminal<TestBackend>, session: &Session) {
    if let Err(error) = terminal.draw(|frame| draw::screen(frame, session)) {
        match error {}
    }
}
