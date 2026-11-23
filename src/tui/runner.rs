use super::model::Session;
use super::post::Mode;
use crate::error::Error;
use crate::gh::Gh;
use crate::git::Git;
use crate::lock::{self, Role};
use crate::notify::{self, Notice};
use crate::store::Store;

pub(super) type PaintRunner<'a> = dyn Fn(&mut Session, bool) -> Result<(), Error> + 'a;

pub(crate) async fn serve<H: Gh>(store: &Store, gh: &H, max_retries: u32) -> Result<(), Error> {
    while crate::publish::serve_next(store, gh, max_retries).await? {}
    Ok(())
}

async fn needs_runner(store: &Store) -> Result<bool, Error> {
    let now = crate::store::now_ms();
    store
        .call(move |conn| {
            conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM jobs WHERE kind IN ('publish','reconcile','sync')
             AND (state = 'running' OR (state = 'queued' AND held = 0
             AND (next_attempt_at IS NULL OR next_attempt_at <= ?1))))",
                [now],
                |row| row.get(0),
            )
        })
        .await
}

pub(super) async fn drive<H: Gh, G: Git>(
    session: &mut Session,
    store: &Store,
    mode: &Mode<'_, H, G>,
) -> Result<(), Error> {
    drive_with(session, store, mode, notify::deliver).await
}

pub(super) async fn drive_with<H: Gh, G: Git>(
    session: &mut Session,
    store: &Store,
    mode: &Mode<'_, H, G>,
    deliver: impl AsyncFn(&Notice) -> Result<(), Error>,
) -> Result<(), Error> {
    let Mode::On {
        gh,
        git,
        paths,
        settings,
        paint_runner,
    } = *mode
    else {
        return Ok(());
    };
    if !needs_runner(store).await? {
        return Ok(());
    }
    let Some(lock) = lock::try_acquire(session.lock_path()).await? else {
        return Ok(());
    };
    lock.announce(Role::Tui).await?;
    let store = store.under_lock(lock);
    if let Some(paint) = paint_runner {
        paint(session, true)?;
    }
    let report = crate::recovery::run(paths, &store, git, settings).await?;
    for notice in notify::recovery::notices(&store, &report.jobs).await? {
        if let Err(error) = deliver(&notice).await {
            session.set_notice(format!("notification-failed: {error}"));
        }
    }
    serve(&store, gh, settings.max_retries()).await?;
    drop(store);
    if let Some(paint) = paint_runner {
        paint(session, false)?;
    }
    Ok(())
}
