use super::fake::Fake;
use super::publish::paths;
use super::queue::{self, HEAD, open};
use crate::config::Settings;
use crate::store::Store;
use crate::tui::model::Session;
use crate::tui::post::Mode;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;

async fn stamp(store: &Store) -> Option<i64> {
    store
        .call(|conn| {
            conn.query_row(
                "SELECT next_attempt_at FROM jobs WHERE kind = 'sync'",
                [],
                |row| row.get(0),
            )
        })
        .await
        .unwrap()
}

#[tokio::test]
async fn the_open_tui_serves_a_sync_after_the_lock_frees() {
    let (tmp, store) = open().await;
    let now = crate::store::now_ms();
    let sql = format!(
        "INSERT INTO targets (id, owner, repo, number, tracking, created_at, head_sha, head_checked_at) VALUES (1, 'o', 'r', 1, 'tracked', 1, '{HEAD}', {now});
         INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (1, 'review', 1, 'succeeded', 1, 0, 0, 0);
         INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (1, 1, 1, 'succeeded', 'findings', '{HEAD}', 'base', 'base', 'main', 0);
         INSERT INTO drafts (id, run_id, original_output, created_at) VALUES (1, 1, '{{}}', 0);
         INSERT INTO publications (id, target_id, draft_id, run_id, state, event, confirmed_sha, snapshot, github_review_id, created_at, updated_at) VALUES ('0123456789abcdef0123456789abcdef', 1, 1, 1, 'submitted', 'COMMENT', '{HEAD}', '{{}}', '9001', 1, 1);
         INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at, publication_id) VALUES (2, 'sync', 1, 'queued', 1, 0, 2, 2, '0123456789abcdef0123456789abcdef');"
    );
    store
        .call(move |conn| conn.execute_batch(&sql))
        .await
        .unwrap();
    let paths = paths(&tmp);
    let mut session = Session::load(&store).await.unwrap();
    session.bind_runner(
        crate::app::ProcessInfo {
            pid: 7,
            started_at_ms: 8,
        },
        paths.lock(),
    );
    let gh = Fake::new(HEAD);
    let git = crate::git::GitCli::new(&crate::procenv::InheritedEnv::from_pairs(Vec::new()));
    let settings = Settings::default();
    let mode = Mode::On {
        gh: &gh,
        git: &git,
        paths: &paths,
        settings: &settings,
    };
    let held = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(paths.lock())
        .unwrap();
    held.try_lock().unwrap();
    super::super::service(&mut session, &store, &mode)
        .await
        .unwrap();
    assert_eq!(stamp(&store).await, None);
    drop(held);
    super::super::service(&mut session, &store, &mode)
        .await
        .unwrap();
    let first = stamp(&store).await.expect("sync was not serviced");
    super::super::service(&mut session, &store, &mode)
        .await
        .unwrap();
    assert_eq!(stamp(&store).await, Some(first));
    store
        .call(|conn| {
            conn.execute(
                "UPDATE jobs SET next_attempt_at = 0 WHERE kind = 'sync'",
                [],
            )
        })
        .await
        .unwrap();
    super::super::service(&mut session, &store, &mode)
        .await
        .unwrap();
    let retried = stamp(&store).await.expect("scheduled sync was not retried");
    assert_ne!(retried, first);
    assert_eq!(
        queue::count(&store, "SELECT count(*) FROM jobs WHERE kind = 'sync'").await,
        1
    );
}

#[tokio::test]
async fn the_open_queue_reloads_when_no_job_is_due() {
    let (tmp, store) = open().await;
    queue::seed(&store, 1).await;
    let paths = paths(&tmp);
    let mut session = Session::load(&store).await.unwrap();
    session.bind_runner(
        crate::app::ProcessInfo {
            pid: 7,
            started_at_ms: 8,
        },
        paths.lock(),
    );
    let gh = Fake::new(HEAD);
    let git = crate::git::GitCli::new(&crate::procenv::InheritedEnv::from_pairs(Vec::new()));
    let settings = Settings::default();
    let mode = Mode::On {
        gh: &gh,
        git: &git,
        paths: &paths,
        settings: &settings,
    };
    super::super::service(&mut session, &store, &mode)
        .await
        .unwrap();
    queue::seed(&store, 2).await;
    super::super::service(&mut session, &store, &mode)
        .await
        .unwrap();
    assert_eq!(session.rows.len(), 2);
    assert_eq!(session.rows[0].target_id, 2);
}

#[tokio::test]
async fn a_moved_head_is_shown_without_resetting_the_choice() {
    let (tmp, store) = open().await;
    queue::seed(&store, 1).await;
    let seen = "b".repeat(40);
    let sql = format!(
        "INSERT INTO publications (id, target_id, draft_id, run_id, state, event, confirmed_sha, snapshot, created_at, updated_at) VALUES ('0123456789abcdef0123456789abcdef', 1, 1, 1, 'pending', 'COMMENT', '{HEAD}', '{{}}', 1, 1);
         INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at, publication_id, head_sha) VALUES (9, 'publish', 1, 'needs_confirmation', 1, 0, 2, 2, '0123456789abcdef0123456789abcdef', '{seen}');"
    );
    store
        .call(move |conn| conn.execute_batch(&sql))
        .await
        .unwrap();
    let paths = paths(&tmp);
    let mut session = Session::load(&store).await.unwrap();
    session.bind_runner(
        crate::app::ProcessInfo {
            pid: 7,
            started_at_ms: 8,
        },
        paths.lock(),
    );
    let gh = Fake::new(HEAD);
    let git = crate::git::GitCli::new(&crate::procenv::InheritedEnv::from_pairs(Vec::new()));
    let settings = Settings::default();
    let mode = Mode::On {
        gh: &gh,
        git: &git,
        paths: &paths,
        settings: &settings,
    };
    super::super::service(&mut session, &store, &mode)
        .await
        .unwrap();
    assert!(session.confirming());
    assert!(session.confirm_modal().unwrap().highlighted_safe);
    session.highlight_confirm(false);
    super::super::service(&mut session, &store, &mode)
        .await
        .unwrap();
    assert!(!session.confirm_modal().unwrap().highlighted_safe);
    session.cancel_confirm();
    super::super::apply(
        &mut session,
        &store,
        &mode,
        queue::key(ratatui::crossterm::event::KeyCode::Enter),
    )
    .await
    .unwrap();
    assert!(session.confirming());
    assert!(session.confirm_modal().unwrap().highlighted_safe);
}
