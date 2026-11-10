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
) -> Result<(), Error> {
    let Mode::On { gh, .. } = *mode else {
        return Ok(());
    };
    let Some(intent) = session.intent() else {
        return Ok(());
    };
    if !intent.undecided.is_empty() {
        let ids = intent.undecided.join(" ");
        session.set_notice(format!("undecided findings: {ids}"));
        return Ok(());
    }
    let head = match gh.pr_view(&intent.pr).await {
        Ok(meta) => meta.head_sha,
        Err(error) => {
            session.set_notice(error.to_string());
            return Ok(());
        }
    };
    store.set_known_head(intent.target, &head).await?;
    let run = run_head(store, intent.draft).await?;
    if head != run {
        let now = crate::store::now_ms();
        let line = crate::status::freshness::stored(&run, Some((head.as_str(), now)), now);
        session.ask_stale(event, &head, line);
        return Ok(());
    }
    publish_now(session, store, mode, event, &head).await
}

pub(super) async fn commit<H: Gh, G: Git>(
    session: &mut Session,
    store: &Store,
    mode: &Mode<'_, H, G>,
    offer: &super::confirm::Offer,
) -> Result<(), Error> {
    publish_now(session, store, mode, &offer.event, &offer.sha).await
}

async fn publish_now<H: Gh, G: Git>(
    session: &mut Session,
    store: &Store,
    mode: &Mode<'_, H, G>,
    event: &str,
    sha: &str,
) -> Result<(), Error> {
    let Mode::On {
        gh,
        git,
        paths,
        settings,
    } = *mode
    else {
        return Ok(());
    };
    let Some(intent) = session.intent() else {
        return Ok(());
    };
    let process = ProcessInfo {
        pid: intent.pid,
        started_at_ms: intent.started,
    };
    match store
        .confirm_publication(intent.draft, event, sha, &process)
        .await
    {
        Ok(_) => {
            runner::drive(session, store, gh, git, paths, settings, &intent).await?;
            session.reload_rows(store).await
        }
        Err(error) => {
            session.set_notice(refusal(&error));
            Ok(())
        }
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

fn refusal(error: &ConfirmError) -> String {
    match error {
        ConfirmError::Undecided(ids) => format!("undecided findings: {}", ids.join(" ")),
        ConfirmError::DraftSubmitted | ConfirmError::Blocked | ConfirmError::Database(_) => {
            error.to_string()
        }
    }
}
