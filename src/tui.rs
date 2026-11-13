mod confirm;
mod draw;
mod edit;
mod keys;
mod model;
mod open;
mod post;
mod reader;
mod runner;
mod spawn;

#[cfg(test)]
mod tests;
#[cfg(test)]
pub(crate) use runner::serve;
#[cfg(test)]
pub(crate) use tests::fake::Fake;

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

async fn until_shutdown(
    signals: &mut crate::review::signals::Signals,
    work: impl std::future::Future<Output = Result<(), Error>>,
    reap: &spawn::ReapGate,
) -> Result<(), Error> {
    tokio::select! {
        biased;
        _ = signals.next(crate::cancel::StopReason::Shutdown) => {
            reap.drain().await;
            Ok(())
        }
        result = work => result,
    }
}

async fn show(config: &AppConfig) -> Result<(), Error> {
    let store = Store::open(&config.paths).await?;
    let mut terminal = ratatui::try_init().map_err(Error::io("open the terminal"))?;
    let _restore = Restore;
    let mut signals = crate::review::signals::Signals::new()?;
    let mut session = Session::load(&store).await?;
    let editor = config
        .env
        .get("EDITOR")
        .and_then(|value| value.to_str())
        .map(str::to_owned);
    session.prepare(editor, config.paths.cache_dir().to_path_buf(), true);
    session.bind_runner(
        crate::app::ProcessInfo {
            pid: config.process.pid,
            started_at_ms: config.process.started_at_ms,
        },
        config.paths.lock(),
    );
    let gh = crate::gh::GhCli::new(&config.env);
    let git = crate::git::GitCli::new(&config.env);
    let mode = post::Mode::On {
        gh: &gh,
        git: &git,
        paths: &config.paths,
        settings: &config.settings,
    };
    let (tx, mut keys) = tokio::sync::mpsc::channel(16);
    let gate = std::sync::Arc::new(reader::KeyGate::new());
    let reader_gate = std::sync::Arc::clone(&gate);
    let _reader = tokio::task::spawn_blocking(move || read_keys(&tx, &reader_gate));
    let reap = std::sync::Arc::clone(&session.reap);
    until_shutdown(
        &mut signals,
        async {
            loop {
                if session.take_repaint() {
                    terminal.clear().map_err(Error::io("clear the terminal"))?;
                }
                draw_session(&mut terminal, &mut session)?;
                let seen = session.draft_id();
                match finish_or_quit(service(&mut session, &store, &mode), &mut keys).await? {
                    Turn::Quit => return Ok(()),
                    Turn::Finished(mut queued) => {
                        take_ready(&mut keys, &mut queued)?;
                        draw_session(&mut terminal, &mut session)?;
                        if drive_queued(
                            &mut session,
                            &store,
                            &mode,
                            seen,
                            queued,
                            Some((&gate, &mut keys)),
                        )
                        .await?
                        {
                            return Ok(());
                        }
                    }
                }
            }
        },
        &reap,
    )
    .await
}

const SERVICE_TICK: std::time::Duration = std::time::Duration::from_secs(1);

fn draw_session<B>(terminal: &mut ratatui::Terminal<B>, session: &mut Session) -> Result<(), Error>
where
    B: ratatui::backend::Backend<Error = io::Error>,
{
    let window = std::cell::Cell::new(draw::Window::empty());
    terminal
        .draw(|frame| window.set(draw::screen(frame, session)))
        .map_err(Error::io("draw the terminal"))?;
    let window = window.into_inner();
    session.note_window(window.width, window.height);
    Ok(())
}

async fn service<H: crate::gh::Gh, G: crate::git::Git>(
    session: &mut Session,
    store: &Store,
    mode: &post::Mode<'_, H, G>,
) -> Result<(), Error> {
    let post::Mode::On {
        gh,
        git,
        paths,
        settings,
    } = *mode
    else {
        return Ok(());
    };
    if runner::needs_runner(store, session.running()).await? {
        runner::drive(session, store, gh, git, paths, settings).await?;
    }
    session.reload_rows(store).await?;
    post::poll_moved(session, store).await?;
    session.reload(store).await
}

type LiveReader<'a> = (
    &'a std::sync::Arc<reader::KeyGate>,
    &'a mut tokio::sync::mpsc::Receiver<Result<Option<KeyEvent>, Error>>,
);

async fn apply<H: crate::gh::Gh, G: crate::git::Git>(
    session: &mut Session,
    store: &Store,
    mode: &post::Mode<'_, H, G>,
    key: KeyEvent,
    reader: Option<LiveReader<'_>>,
) -> Result<bool, Error> {
    session.clear_notice();
    match keys::on_key(session, key) {
        keys::Command::Quit => Ok(true),
        keys::Command::Open => {
            session.open_draft(store).await?;
            post::poll_moved(session, store).await?;
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
        keys::Command::Edit => edit_key(session, store, reader).await,
        keys::Command::Include => {
            session.include_limitation(store).await?;
            Ok(false)
        }
        keys::Command::Discard => {
            session.ask_discard();
            Ok(false)
        }
        keys::Command::OpenPr => {
            session.open_pull().await?;
            Ok(false)
        }
        keys::Command::Rerun => {
            session.rerun(store).await?;
            Ok(false)
        }
        keys::Command::Unpublish => {
            session.ask_cancel(store).await?;
            Ok(false)
        }
        keys::Command::Reconcile => post::reconcile(session, store, mode, reader).await,
        keys::Command::Abandon => {
            session.ask_abandon(store).await?;
            Ok(false)
        }
        keys::Command::Approve => post::send(session, store, mode, "APPROVE", reader).await,
        keys::Command::Request => post::send(session, store, mode, "REQUEST_CHANGES", reader).await,
        keys::Command::Comment => post::send(session, store, mode, "COMMENT", reader).await,
        keys::Command::Cancel => {
            session.cancel_confirm();
            Ok(false)
        }
        keys::Command::Accept => {
            if let Some(offer) = session.accept_confirm(store).await? {
                return post::commit(session, store, mode, &offer, reader).await;
            }
            Ok(false)
        }
        keys::Command::Safer => {
            session.highlight_confirm(true);
            Ok(false)
        }
        keys::Command::Riskier => {
            session.highlight_confirm(false);
            Ok(false)
        }
        keys::Command::Ignored => Ok(false),
    }
}

async fn edit_key(
    session: &mut Session,
    store: &Store,
    reader: Option<LiveReader<'_>>,
) -> Result<bool, Error> {
    if let Some((gate, keys)) = reader.filter(|_| session.edits_text()) {
        pause_for_editor(gate, keys, async {
            let edited = session.edit_selected(store).await;
            discard_pending_keys();
            edited
        })
        .await?;
    } else {
        session.edit_selected(store).await?;
    }
    Ok(false)
}

struct Restore;

impl Drop for Restore {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

#[derive(Debug)]
enum Turn {
    Finished(Vec<KeyEvent>),
    Quit,
}

async fn finish_or_quit<F>(
    work: F,
    keys: &mut tokio::sync::mpsc::Receiver<Result<Option<KeyEvent>, Error>>,
) -> Result<Turn, Error>
where
    F: std::future::Future<Output = Result<(), Error>>,
{
    tokio::pin!(work);
    let mut queued = Vec::new();
    loop {
        tokio::select! {
            biased;
            result = &mut work => {
                result?;
                return Ok(Turn::Finished(queued));
            }
            incoming = keys.recv() => {
                if let Some(key) = incoming.ok_or_else(key_stopped)?? {
                    if keys::quits(&key) {
                        return Ok(Turn::Quit);
                    }
                    queued.push(key);
                }
            }
        }
    }
}

async fn drive_queued<H: crate::gh::Gh, G: crate::git::Git>(
    session: &mut Session,
    store: &Store,
    mode: &post::Mode<'_, H, G>,
    seen: Option<i64>,
    queued: Vec<KeyEvent>,
    mut reader: Option<LiveReader<'_>>,
) -> Result<bool, Error> {
    let idle = queued.is_empty();
    let changed = seen.is_some() && session.draft_id() != seen;
    for key in queued {
        if changed && keys::publishes(&key) {
            session.set_notice("the open draft changed".to_string());
            continue;
        }
        if apply(session, store, mode, key, live(&mut reader)).await? {
            return Ok(true);
        }
    }
    if idle
        && let Some(key) = next_ready(&mut reader).await?
        && apply(session, store, mode, key, live(&mut reader)).await?
    {
        return Ok(true);
    }
    Ok(false)
}

fn live<'short, 'long: 'short>(
    reader: &'short mut Option<LiveReader<'long>>,
) -> Option<LiveReader<'short>> {
    reader.as_mut().map(|(gate, keys)| (*gate, &mut **keys))
}

async fn next_ready(reader: &mut Option<LiveReader<'_>>) -> Result<Option<KeyEvent>, Error> {
    let Some((_, keys)) = reader.as_mut() else {
        return Ok(None);
    };
    next_key(keys).await
}

fn take_ready(
    keys: &mut tokio::sync::mpsc::Receiver<Result<Option<KeyEvent>, Error>>,
    queued: &mut Vec<KeyEvent>,
) -> Result<(), Error> {
    loop {
        match keys.try_recv() {
            Ok(Ok(Some(key))) => queued.push(key),
            Ok(Ok(None)) => {}
            Ok(Err(error)) => return Err(error),
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => return Ok(()),
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                return Err(key_stopped());
            }
        }
    }
}

async fn next_key(
    keys: &mut tokio::sync::mpsc::Receiver<Result<Option<KeyEvent>, Error>>,
) -> Result<Option<KeyEvent>, Error> {
    incoming_key(keys.recv().await)
}

fn incoming_key(
    incoming: Option<Result<Option<KeyEvent>, Error>>,
) -> Result<Option<KeyEvent>, Error> {
    incoming.ok_or_else(key_stopped)?
}

fn key_stopped() -> Error {
    Error::io("read a key")(io::Error::other("key reader stopped"))
}

// crossterm::event::poll blocks in the kernel and cannot be cancelled.
fn read_keys(
    tx: &tokio::sync::mpsc::Sender<Result<Option<KeyEvent>, Error>>,
    gate: &reader::KeyGate,
) {
    pump(tx, gate, read_one);
}

fn pump(
    tx: &tokio::sync::mpsc::Sender<Result<Option<KeyEvent>, Error>>,
    gate: &reader::KeyGate,
    mut read_one: impl FnMut() -> Result<Option<KeyEvent>, Error>,
) {
    loop {
        match gate.checkpoint() {
            reader::Step::Stop => break,
            reader::Step::Skip => continue,
            reader::Step::Run => {}
        }
        let event = read_one();
        match gate.checkpoint() {
            reader::Step::Stop => break,
            reader::Step::Skip => continue,
            reader::Step::Run => {}
        }
        let failed = event.is_err();
        if tx.blocking_send(event).is_err() || failed {
            gate.stop();
            break;
        }
    }
}

async fn pause_for_editor<T>(
    gate: &std::sync::Arc<reader::KeyGate>,
    keys: &mut tokio::sync::mpsc::Receiver<Result<Option<KeyEvent>, Error>>,
    edit: impl std::future::Future<Output = T>,
) -> T {
    let mut stop = StopGate(Some(gate));
    let waiting = std::sync::Arc::clone(gate);
    let pause = tokio::task::spawn_blocking(move || waiting.suspend_and_wait());
    tokio::pin!(pause);
    let mut closed = false;
    loop {
        tokio::select! {
            biased;
            joined = &mut pause => {
                let _ = joined;
                break;
            }
            incoming = keys.recv(), if !closed => {
                closed = incoming.is_none();
            }
        }
    }
    while keys.try_recv().is_ok() {}
    stop.0 = None;
    let _resume = Resume(gate);
    edit.await
}

struct StopGate<'a>(Option<&'a reader::KeyGate>);

impl Drop for StopGate<'_> {
    fn drop(&mut self) {
        if let Some(gate) = self.0.take() {
            gate.stop();
        }
    }
}

struct Resume<'a>(&'a reader::KeyGate);

impl Drop for Resume<'_> {
    fn drop(&mut self) {
        self.0.resume();
    }
}

fn discard_pending_keys() {
    let poll = ratatui::crossterm::event::poll;
    let read = ratatui::crossterm::event::read;
    while poll(std::time::Duration::ZERO).unwrap_or(false) {
        if read().is_err() {
            break;
        }
    }
}

fn read_one() -> Result<Option<KeyEvent>, Error> {
    if !ratatui::crossterm::event::poll(SERVICE_TICK).map_err(Error::io("read a key"))? {
        return Ok(None);
    }
    loop {
        if let Event::Key(key) =
            ratatui::crossterm::event::read().map_err(Error::io("read a key"))?
            && key.kind == KeyEventKind::Press
        {
            return Ok(Some(key));
        }
        if !ratatui::crossterm::event::poll(std::time::Duration::ZERO)
            .map_err(Error::io("read a key"))?
        {
            return Ok(None);
        }
    }
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
    draw_scripted(&mut terminal, &mut session);
    for key in keys {
        if apply(&mut session, store, &idle(), *key, None).await? {
            break;
        }
        draw_scripted(&mut terminal, &mut session);
    }
    Ok(terminal.backend().clone())
}

#[cfg(test)]
pub(crate) async fn run_scripted_actions(
    store: &Store,
    keys: &[KeyEvent],
    backend: TestBackend,
    lock: Option<&std::path::Path>,
    opener: Option<&str>,
) -> Result<TestBackend, Error> {
    let mut terminal = Terminal::new(backend).unwrap_or_else(|error| match error {});
    let mut session = Session::load(store).await?;
    session.arm_actions(
        lock.map(std::path::Path::to_path_buf),
        opener.map(str::to_owned),
    );
    draw_scripted(&mut terminal, &mut session);
    for key in keys {
        if apply(&mut session, store, &idle(), *key, None).await? {
            break;
        }
        draw_scripted(&mut terminal, &mut session);
    }
    Ok(terminal.backend().clone())
}

#[cfg(test)]
pub(crate) async fn run_scripted_publish<H: crate::gh::Gh, G: crate::git::Git>(
    store: &Store,
    keys: &[KeyEvent],
    backend: TestBackend,
    paths: &crate::paths::Paths,
    gh: &H,
    git: &G,
    settings: &crate::config::Settings,
) -> Result<TestBackend, Error> {
    let mut terminal = Terminal::new(backend).unwrap_or_else(|error| match error {});
    let mut session = Session::load(store).await?;
    session.bind_runner(
        crate::app::ProcessInfo {
            pid: 7,
            started_at_ms: 8,
        },
        paths.lock(),
    );
    let mode = post::Mode::On {
        gh,
        git,
        paths,
        settings,
    };
    draw_scripted(&mut terminal, &mut session);
    for key in keys {
        if apply(&mut session, store, &mode, *key, None).await? {
            break;
        }
        draw_scripted(&mut terminal, &mut session);
    }
    Ok(terminal.backend().clone())
}

#[cfg(test)]
fn idle() -> post::Mode<'static, crate::gh::GhCli, crate::git::GitCli> {
    post::Mode::Off
}

#[cfg(test)]
fn draw_scripted(terminal: &mut Terminal<TestBackend>, session: &mut Session) {
    let window = std::cell::Cell::new(draw::Window::empty());
    if let Err(error) = terminal.draw(|frame| window.set(draw::screen(frame, session))) {
        match error {}
    }
    let window = window.into_inner();
    session.note_window(window.width, window.height);
}
