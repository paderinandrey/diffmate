use super::fake::Fake;
use super::fixture::{self, cell, open_session};
use super::publish::paths;
use super::queue::{key, open};
use crate::config::Settings;
use crate::git::GitCli;
use crate::procenv::InheritedEnv;
use crate::store::Store;
use ratatui::crossterm::event::KeyCode;

async fn moved(store: &Store, seen: &str) {
    fixture::plant(store, "pending", "needs_confirmation").await;
    let seen = seen.to_string();
    store
        .call(move |conn| conn.execute("UPDATE jobs SET head_sha = ?1 WHERE id = 2", [seen]))
        .await
        .unwrap();
}

#[tokio::test]
async fn send_anyway_from_the_queue_resumes_the_publication() {
    let seen = "b".repeat(40);
    let (tmp, store) = open().await;
    moved(&store, &seen).await;
    let paths = paths(&tmp);
    let mut session = open_session(&store, &paths.lock()).await;
    let gh = Fake::new(&seen);
    let git = GitCli::new(&InheritedEnv::from_pairs(Vec::new()));
    let settings = Settings::default();
    let mode = fixture::mode(&gh, &git, &paths, &settings);
    super::super::service(&mut session, &store, &mode)
        .await
        .unwrap();
    assert!(session.draft_id().is_none());
    assert!(session.confirming());
    session.highlight_confirm(false);
    super::super::apply(&mut session, &store, &mode, key(KeyCode::Enter), None)
        .await
        .unwrap();
    assert_ne!(session.notice(), Some("the open draft changed"));
    assert_eq!(
        cell(&store, "SELECT state FROM publications").await,
        "submitted"
    );
    assert_eq!(gh.seen().len(), 1);
}

#[tokio::test]
async fn a_newer_queue_draft_refuses_the_old_confirmation() {
    let seen = "b".repeat(40);
    let (tmp, store) = open().await;
    moved(&store, &seen).await;
    let paths = paths(&tmp);
    let mut session = open_session(&store, &paths.lock()).await;
    let gh = Fake::new(&seen);
    let git = GitCli::new(&InheritedEnv::from_pairs(Vec::new()));
    let settings = Settings::default();
    let mode = fixture::mode(&gh, &git, &paths, &settings);
    super::super::service(&mut session, &store, &mode)
        .await
        .unwrap();
    let head = "c".repeat(40);
    let recorded = head.clone();
    store
        .call(move |conn| {
            conn.execute_batch(&format!(
                "INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (3, 'review', 1, 'succeeded', 1, 0, 3, 3);
                 INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (2, 1, 3, 'succeeded', 'no_findings', '{recorded}', 'base', 'base', 'main', 1);
                 INSERT INTO drafts (id, run_id, original_output, created_at) VALUES (2, 2, '{{}}', 1);"
            ))
        })
        .await
        .unwrap();
    session.highlight_confirm(false);
    super::super::apply(&mut session, &store, &mode, key(KeyCode::Enter), None)
        .await
        .unwrap();
    assert_eq!(session.notice(), Some("the open draft changed"));
    assert_eq!(
        cell(&store, "SELECT state FROM jobs WHERE id = 2").await,
        "needs_confirmation"
    );
    assert_eq!(gh.seen().len(), 0);
    super::super::service(&mut session, &store, &mode)
        .await
        .unwrap();
    assert!(session.confirming());
    super::super::apply(&mut session, &store, &mode, key(KeyCode::Enter), None)
        .await
        .unwrap();
    assert_eq!(
        cell(&store, "SELECT state FROM jobs WHERE id = 2").await,
        "cancelled"
    );
    assert_eq!(
        cell(&store, "SELECT state FROM publications").await,
        "abandoned"
    );
    let process = session.process_info();
    store
        .confirm_publication(2, "COMMENT", &head, &process)
        .await
        .unwrap();
}
