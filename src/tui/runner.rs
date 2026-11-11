use super::model::Session;
use crate::config::Settings;
use crate::error::Error;
use crate::gh::Gh;
use crate::git::Git;
use crate::lock::{self, Role};
use crate::paths::Paths;
use crate::store::Store;
use rusqlite::OptionalExtension;

pub(crate) async fn serve<H: Gh>(store: &Store, gh: &H, max_retries: u32) -> Result<(), Error> {
    loop {
        let Some((id, kind)) = next_service(store).await? else {
            return Ok(());
        };
        match kind.as_str() {
            "publish" => {
                store.start_job(id).await?;
                crate::publish::execute_publish(store, gh, id, max_retries).await?;
            }
            "reconcile" => {
                store.start_job(id).await?;
                crate::publish::execute_reconcile(store, gh, id).await?;
            }
            "sync" => {
                store.start_job(id).await?;
                crate::publish::execute_sync(store, gh, id).await?;
            }
            _ => return Ok(()),
        }
    }
}

pub(super) async fn drive<H: Gh, G: Git>(
    session: &mut Session,
    store: &Store,
    gh: &H,
    git: &G,
    paths: &Paths,
    settings: &Settings,
) -> Result<(), Error> {
    if !session.running() {
        let Some(lock) = lock::try_acquire(session.lock_path()).await? else {
            return Ok(());
        };
        lock.announce(Role::Tui).await?;
        session.hold(lock);
        crate::recovery::run(paths, store, git, settings).await?;
    }
    serve(store, gh, settings.max_retries()).await
}

async fn next_service(store: &Store) -> Result<Option<(i64, String)>, Error> {
    let now = crate::store::now_ms();
    store
        .call(move |conn| {
            let sql = "SELECT id, kind FROM jobs WHERE kind IN ('publish', 'reconcile', 'sync') \
                       AND state = 'queued' AND held = 0 \
                       AND (next_attempt_at IS NULL OR next_attempt_at <= ?1) \
                       ORDER BY id LIMIT 1";
            conn.query_row(sql, [now], |row| Ok((row.get(0)?, row.get(1)?)))
                .optional()
        })
        .await
}
