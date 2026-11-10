use super::model::{Intent, Session};
use crate::config::Settings;
use crate::error::Error;
use crate::gh::Gh;
use crate::git::Git;
use crate::lock::{self, Role};
use crate::paths::Paths;
use crate::store::Store;
use rusqlite::OptionalExtension;

pub(super) async fn serve<H: Gh>(store: &Store, gh: &H) -> Result<(), Error> {
    loop {
        let Some((id, kind)) = next_service(store).await? else {
            return Ok(());
        };
        if kind != "publish" {
            return Ok(());
        }
        store.start_job(id).await?;
        crate::publish::execute_publish(store, gh, id).await?;
    }
}

pub(super) async fn drive<H: Gh, G: Git>(
    session: &mut Session,
    store: &Store,
    gh: &H,
    git: &G,
    paths: &Paths,
    settings: &Settings,
    intent: &Intent,
) -> Result<(), Error> {
    if !session.running() {
        let Some(lock) = lock::try_acquire(&intent.lock).await? else {
            return Ok(());
        };
        lock.announce(Role::Tui).await?;
        session.hold(lock);
        crate::recovery::run(paths, store, git, settings).await?;
    }
    serve(store, gh).await
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
