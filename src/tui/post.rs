use super::model::Session;
use super::runner;
use crate::app::ProcessInfo;
use crate::config::Settings;
use crate::error::Error;
use crate::gh::Gh;
use crate::git::Git;
use crate::paths::Paths;
use crate::store::Store;
use crate::store::publish::ConfirmError;
use rusqlite::OptionalExtension;

#[derive(Clone, Copy)]
pub(super) enum Mode<'a, H, G> {
    #[cfg_attr(not(test), allow(dead_code))]
    Off,
    On {
        gh: &'a H,
        git: &'a G,
        paths: &'a Paths,
        settings: &'a Settings,
    },
}

pub(super) async fn send<H: Gh, G: Git>(
    session: &mut Session,
    store: &Store,
    mode: &Mode<'_, H, G>,
    event: &str,
    mut reader: Option<super::LiveReader<'_>>,
) -> Result<bool, Error> {
    let Mode::On { gh, .. } = *mode else {
        return Ok(false);
    };
    let Some(intent) = session.intent() else {
        return Ok(false);
    };
    if !intent.undecided.is_empty() {
        let ids = intent.undecided.join(" ");
        session.set_notice(format!("undecided findings: {ids}"));
        return Ok(false);
    }
    let head = match read_head(gh, &intent.pr, &mut reader).await? {
        Head::Quit => return Ok(true),
        Head::Failed(error) => {
            session.set_notice(error.to_string());
            return Ok(false);
        }
        Head::Sha(sha) => sha,
    };
    store.set_known_head(intent.target, &head).await?;
    let run = run_head(store, intent.draft).await?;
    if head != run {
        let now = crate::store::now_ms();
        let line = crate::status::freshness::stored(&run, Some((head.as_str(), now)), now);
        session.ask_stale(intent.draft, event, &head, line);
        return Ok(false);
    }
    publish_now(session, store, mode, intent.draft, event, &head, reader).await
}

pub(super) async fn commit<H: Gh, G: Git>(
    session: &mut Session,
    store: &Store,
    mode: &Mode<'_, H, G>,
    offer: &super::confirm::Offer,
    reader: Option<super::LiveReader<'_>>,
) -> Result<bool, Error> {
    if offer.publication.is_some() {
        return resume(session, store, mode, offer, reader).await;
    }
    publish_now(
        session,
        store,
        mode,
        offer.draft,
        &offer.event,
        &offer.sha,
        reader,
    )
    .await
}

async fn publish_now<H: Gh, G: Git>(
    session: &mut Session,
    store: &Store,
    mode: &Mode<'_, H, G>,
    draft: i64,
    event: &str,
    sha: &str,
    reader: Option<super::LiveReader<'_>>,
) -> Result<bool, Error> {
    let Mode::On {
        gh,
        git,
        paths,
        settings,
    } = *mode
    else {
        return Ok(false);
    };
    if replaced(session, store, draft).await? {
        return Ok(false);
    }
    let Some(intent) = session.intent() else {
        return Ok(false);
    };
    let process = ProcessInfo {
        pid: intent.pid,
        started_at_ms: intent.started,
    };
    match store.confirm_publication(draft, event, sha, &process).await {
        Ok(_) => {
            if drive_until_quit(session, store, gh, git, paths, settings, reader).await? {
                return Ok(true);
            }
            surface(session, store, draft).await?;
            session.reload_rows(store).await?;
            Ok(false)
        }
        Err(error) => {
            session.set_notice(refusal(&error));
            Ok(false)
        }
    }
}

async fn resume<H: Gh, G: Git>(
    session: &mut Session,
    store: &Store,
    mode: &Mode<'_, H, G>,
    offer: &super::confirm::Offer,
    reader: Option<super::LiveReader<'_>>,
) -> Result<bool, Error> {
    let Mode::On {
        gh,
        git,
        paths,
        settings,
    } = *mode
    else {
        return Ok(false);
    };
    if replaced(session, store, offer.draft).await? {
        return Ok(false);
    }
    let Some(id) = offer.publication.clone() else {
        return Ok(false);
    };
    let sha = offer.sha.clone();
    let draft = offer.draft;
    store.reconfirm(&id, &sha).await?;
    if drive_until_quit(session, store, gh, git, paths, settings, reader).await? {
        return Ok(true);
    }
    surface(session, store, draft).await?;
    session.reload_rows(store).await?;
    Ok(false)
}

pub(super) async fn reconcile<H: Gh, G: Git>(
    session: &mut Session,
    store: &Store,
    mode: &Mode<'_, H, G>,
    reader: Option<super::LiveReader<'_>>,
) -> Result<bool, Error> {
    let Mode::On {
        gh,
        git,
        paths,
        settings,
    } = *mode
    else {
        return Ok(false);
    };
    let Some(target) = session.selected_target() else {
        return Ok(false);
    };
    let Some(id) = store.unknown_publication(target).await? else {
        return Ok(false);
    };
    let process = session.process_info();
    store.enqueue_reconcile(&id, &process).await?;
    if drive_until_quit(session, store, gh, git, paths, settings, reader).await? {
        return Ok(true);
    }
    session.reload_rows(store).await?;
    Ok(false)
}

enum Head {
    Sha(String),
    Failed(Error),
    Quit,
}

async fn read_head<H: Gh>(
    gh: &H,
    pr: &crate::pr_url::PrRef,
    reader: &mut Option<super::LiveReader<'_>>,
) -> Result<Head, Error> {
    let view = gh.pr_view(pr);
    let Some((_, keys)) = reader.as_mut() else {
        return match view.await {
            Ok(meta) => Ok(Head::Sha(meta.head_sha)),
            Err(error) => Ok(Head::Failed(error)),
        };
    };
    tokio::pin!(view);
    loop {
        tokio::select! {
            biased;
            result = &mut view => {
                return match result {
                    Ok(meta) => Ok(Head::Sha(meta.head_sha)),
                    Err(error) => Ok(Head::Failed(error)),
                };
            }
            incoming = keys.recv() => {
                let Some(key) = incoming.ok_or_else(super::key_stopped)?? else {
                    continue;
                };
                if super::keys::quits(&key) {
                    return Ok(Head::Quit);
                }
            }
        }
    }
}

async fn drive_until_quit<H: Gh, G: Git>(
    session: &mut Session,
    store: &Store,
    gh: &H,
    git: &G,
    paths: &Paths,
    settings: &Settings,
    reader: Option<super::LiveReader<'_>>,
) -> Result<bool, Error> {
    let drive = runner::drive(session, store, gh, git, paths, settings);
    let Some((_, keys)) = reader else {
        drive.await?;
        return Ok(false);
    };
    match super::finish_or_quit(drive, keys).await? {
        super::Turn::Quit => Ok(true),
        super::Turn::Finished(_) => Ok(false),
    }
}

pub(super) async fn poll_moved(session: &mut Session, store: &Store) -> Result<(), Error> {
    if session.confirming() {
        return Ok(());
    }
    if let Some(draft) = waiting_draft(session, store).await? {
        return surface(session, store, draft).await;
    }
    let draft = if let Some(id) = session.draft_id() {
        id
    } else if let Some(target) = session.selected_target() {
        let Some(draft) = store.latest_draft(target).await? else {
            return Ok(());
        };
        draft.id
    } else {
        return Ok(());
    };
    surface(session, store, draft).await
}

async fn waiting_draft(session: &Session, store: &Store) -> Result<Option<i64>, Error> {
    let Some(target) = session.selected_target() else {
        return Ok(None);
    };
    store
        .call(move |conn| {
            let sql = "SELECT p.draft_id FROM publications p \
                       JOIN jobs j ON j.publication_id = p.id AND j.kind = 'publish' \
                       WHERE p.target_id = ?1 AND p.state = 'pending' \
                       AND j.state = 'needs_confirmation' \
                       ORDER BY j.id DESC LIMIT 1";
            conn.query_row(sql, [target], |row| row.get(0)).optional()
        })
        .await
}

struct Follow {
    id: String,
    state: String,
    reason: Option<String>,
    job: String,
    seen: Option<String>,
    run: String,
}

async fn surface(session: &mut Session, store: &Store, draft: i64) -> Result<(), Error> {
    let Some(row) = follow(store, draft).await? else {
        return Ok(());
    };
    if row.job == "needs_confirmation"
        && row.state == "pending"
        && let Some(seen) = row.seen
    {
        let now = crate::store::now_ms();
        let line = crate::status::freshness::stored(&row.run, Some((seen.as_str(), now)), now);
        session.ask_moved(draft, &row.id, &seen, line);
        return Ok(());
    }
    if row.state == "abandoned"
        && let Some(reason) = row.reason.filter(|text| !text.is_empty())
    {
        session.set_notice(reason);
    }
    Ok(())
}

async fn follow(store: &Store, draft: i64) -> Result<Option<Follow>, Error> {
    store
        .call(move |conn| {
            let sql = "SELECT p.id, p.state, p.reason, j.state, j.head_sha, r.head_sha \
                       FROM publications p \
                       JOIN jobs j ON j.publication_id = p.id AND j.kind = 'publish' \
                       JOIN runs r ON r.id = p.run_id \
                       WHERE p.draft_id = ?1 ORDER BY j.id DESC LIMIT 1";
            conn.query_row(sql, [draft], |row| {
                Ok(Follow {
                    id: row.get(0)?,
                    state: row.get(1)?,
                    reason: row.get(2)?,
                    job: row.get(3)?,
                    seen: row.get(4)?,
                    run: row.get(5)?,
                })
            })
            .optional()
        })
        .await
}

async fn run_head(store: &Store, draft: i64) -> Result<String, Error> {
    store
        .call(move |conn| {
            let sql =
                "SELECT r.head_sha FROM drafts d JOIN runs r ON r.id = d.run_id WHERE d.id = ?1";
            conn.query_row(sql, [draft], |row| row.get(0))
        })
        .await
}

async fn replaced(session: &mut Session, store: &Store, draft: i64) -> Result<bool, Error> {
    if session.draft_id() == Some(draft) {
        return Ok(false);
    }
    if session.draft_id().is_none()
        && let Some(target) = session.selected_target()
        && store
            .latest_draft(target)
            .await?
            .is_some_and(|row| row.id == draft)
    {
        return Ok(false);
    }
    session.set_notice("the open draft changed".to_string());
    Ok(true)
}

fn refusal(error: &ConfirmError) -> String {
    match error {
        ConfirmError::Undecided(ids) => format!("undecided findings: {}", ids.join(" ")),
        ConfirmError::DraftSubmitted | ConfirmError::Blocked | ConfirmError::Database(_) => {
            error.to_string()
        }
    }
}
