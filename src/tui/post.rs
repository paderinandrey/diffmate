use super::confirm::Offer;
use super::model::Session;
use super::runner;
use crate::config::Settings;
use crate::error::Error;
use crate::gh::Gh;
use crate::git::Git;
use crate::paths::Paths;
use crate::store::Store;
use crate::store::publish::ConfirmError;
use crate::store::status::DraftView;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub(super) enum Mode<'a, H, G> {
    #[cfg_attr(not(test), allow(dead_code))]
    Off,
    On {
        gh: &'a H,
        git: &'a G,
        paths: &'a Paths,
        settings: &'a Settings,
        paint_runner: Option<&'a runner::PaintRunner<'a>>,
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
    let run = run_head(store, intent.displayed.id).await?;
    if head != run {
        let now = crate::store::now_ms();
        let line = crate::status::freshness::stored(&run, Some((head.as_str(), now)), now);
        session.ask_stale(intent.displayed, event, &head, line);
        return Ok(false);
    }
    publish_now(session, store, mode, intent.displayed, event, &head, reader).await
}

pub(super) async fn commit<H: Gh, G: Git>(
    session: &mut Session,
    store: &Store,
    mode: &Mode<'_, H, G>,
    offer: Offer,
    reader: Option<super::LiveReader<'_>>,
) -> Result<bool, Error> {
    match offer {
        Offer::New {
            displayed,
            event,
            sha,
        } => publish_now(session, store, mode, displayed, &event, &sha, reader).await,
        Offer::Moved {
            draft,
            publication,
            sha,
        } => resume(session, store, mode, draft, &publication, &sha, reader).await,
    }
}

async fn publish_now<H: Gh, G: Git>(
    session: &mut Session,
    store: &Store,
    mode: &Mode<'_, H, G>,
    displayed: Arc<DraftView>,
    event: &str,
    sha: &str,
    reader: Option<super::LiveReader<'_>>,
) -> Result<bool, Error> {
    if matches!(mode, Mode::Off) {
        return Ok(false);
    }
    if session.draft_id() != Some(displayed.id) {
        session.set_notice("the open draft changed".into());
        return Ok(false);
    }
    match store
        .confirm_publication(displayed, event, sha, &session.process_info())
        .await
    {
        Ok(_) => {
            if drive_until_quit(session, store, mode, reader).await? {
                return Ok(true);
            }
            super::follow::refresh(session, store).await?;
            session.reload_rows(store).await?;
            Ok(false)
        }
        Err(error) => {
            if matches!(error, ConfirmError::Changed) {
                session.reload(store).await?;
            }
            session.set_notice(refusal(&error));
            Ok(false)
        }
    }
}

async fn resume<H: Gh, G: Git>(
    session: &mut Session,
    store: &Store,
    mode: &Mode<'_, H, G>,
    draft: i64,
    id: &str,
    sha: &str,
    reader: Option<super::LiveReader<'_>>,
) -> Result<bool, Error> {
    if matches!(mode, Mode::Off) {
        return Ok(false);
    }
    if replaced(session, store, draft).await? {
        return Ok(false);
    }
    if !store.reconfirm(id, sha).await? {
        session.already_handled();
        session.reload_rows(store).await?;
        return Ok(false);
    }
    if drive_until_quit(session, store, mode, reader).await? {
        return Ok(true);
    }
    super::follow::refresh(session, store).await?;
    session.reload_rows(store).await?;
    Ok(false)
}

pub(super) async fn reconcile<H: Gh, G: Git>(
    session: &mut Session,
    store: &Store,
    mode: &Mode<'_, H, G>,
    reader: Option<super::LiveReader<'_>>,
) -> Result<bool, Error> {
    if matches!(mode, Mode::Off) {
        return Ok(false);
    }
    let Some(target) = session.selected_target() else {
        return Ok(false);
    };
    let Some(id) = store.unknown_publication(target).await? else {
        return Ok(false);
    };
    let process = session.process_info();
    store.enqueue_reconcile(&id, &process).await?;
    if drive_until_quit(session, store, mode, reader).await? {
        return Ok(true);
    }
    session.reload_rows(store).await?;
    super::follow::refresh(session, store).await?;
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
                if super::keys::quits(&key.key) {
                    return Ok(Head::Quit);
                }
            }
        }
    }
}

async fn drive_until_quit<H: Gh, G: Git>(
    session: &mut Session,
    store: &Store,
    mode: &Mode<'_, H, G>,
    reader: Option<super::LiveReader<'_>>,
) -> Result<bool, Error> {
    let drive = runner::drive(session, store, mode);
    let Some((_, keys)) = reader else {
        drive.await?;
        return Ok(false);
    };
    match super::finish_or_quit(drive, keys).await? {
        super::Turn::Quit => Ok(true),
        super::Turn::Finished(_) => Ok(false),
    }
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
        ConfirmError::Changed
        | ConfirmError::DraftSubmitted
        | ConfirmError::Blocked
        | ConfirmError::Database(_) => error.to_string(),
    }
}
