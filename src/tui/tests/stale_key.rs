use super::fake::Fake;
use super::fixture::{self, open_session};
use super::publish::{self, paths};
use super::queue::{self, HEAD, key, open};
use crate::config::Settings;
use crate::paths::Paths;
use crate::store::Store;
use crate::tui::model::Session;
use ratatui::crossterm::event::KeyCode;

async fn shown(store: &Store, lock: &std::path::Path) -> Session {
    let mut session = open_session(store, lock).await;
    session.open_draft(store).await.unwrap();
    session.note_window(80, 8);
    session
}

async fn press(session: &mut Session, store: &Store, paths: &Paths, codes: &[KeyCode]) -> bool {
    let gh = Fake::new(HEAD);
    let git = crate::git::GitCli::new(&crate::procenv::InheritedEnv::from_pairs(Vec::new()));
    let settings = Settings::default();
    let mode = fixture::mode(&gh, &git, paths, &settings);
    let gate = std::sync::Arc::new(super::super::reader::KeyGate::new());
    let (_tx, mut keys) = tokio::sync::mpsc::channel(4);
    super::super::drive_queued(
        session,
        store,
        &mode,
        codes.iter().copied().map(key).map(Into::into).collect(),
        Some((&gate, &mut keys)),
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn queued_approve_drops_when_the_draft_changes() {
    let (tmp, store) = open().await;
    publish::seed(&store).await;
    let paths = paths(&tmp);
    let mut session = shown(&store, &paths.lock()).await;
    let head = HEAD;
    store
        .call(move |conn| {
            conn.execute_batch(&format!(
                "INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (3, 'review', 1, 'succeeded', 1, 0, 3, 3);
                 INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (2, 1, 3, 'succeeded', 'no_findings', '{head}', 'base', 'base', 'main', 3);
                 INSERT INTO drafts (id, run_id, original_output, created_at) VALUES (2, 2, '{{}}', 3);"
            ))
        })
        .await
        .unwrap();
    session.reload(&store).await.unwrap();
    assert_eq!(session.draft_id(), Some(2));
    assert!(!press(&mut session, &store, &paths, &[KeyCode::Char('a')]).await);
    assert_eq!(
        queue::count(&store, "SELECT count(*) FROM publications").await,
        0
    );
    assert_eq!(session.notice(), Some("the open draft changed"));
}

#[tokio::test]
async fn queued_approve_posts_when_the_draft_stays() {
    let (tmp, store) = open().await;
    publish::seed(&store).await;
    let paths = paths(&tmp);
    let mut session = shown(&store, &paths.lock()).await;
    assert_eq!(session.draft_id(), Some(1));
    assert!(!press(&mut session, &store, &paths, &[KeyCode::Char('a')]).await);
    assert_eq!(
        queue::count(&store, "SELECT count(*) FROM publications").await,
        1
    );
}

#[tokio::test]
async fn queued_keep_drops_when_the_draft_changes() {
    let (tmp, store) = open().await;
    publish::seed(&store).await;
    let paths = paths(&tmp);
    let mut session = shown(&store, &paths.lock()).await;
    let head = HEAD;
    store
        .call(move |conn| {
            conn.execute_batch(&format!(
                "INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (3, 'review', 1, 'succeeded', 1, 0, 3, 3);
                 INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (2, 1, 3, 'succeeded', 'findings', '{head}', 'base', 'base', 'main', 3);
                 INSERT INTO drafts (id, run_id, original_output, created_at) VALUES (2, 2, '{{}}', 3);
                 INSERT INTO findings (id, run_id, draft_id, ordinal, severity, title, body) VALUES ('f2', 2, 2, 1, 'low', 'New', 'body');"
            ))
        })
        .await
        .unwrap();
    session.reload(&store).await.unwrap();
    assert!(!press(&mut session, &store, &paths, &[KeyCode::Char('K')]).await);
    let kept = "SELECT count(*) FROM findings WHERE id = 'f2' AND decision = 'kept'";
    assert_eq!(queue::count(&store, kept).await, 0);
    assert_eq!(session.notice(), Some("the open draft changed"));
}

#[tokio::test]
async fn queued_drop_writes_when_the_draft_stays() {
    let (tmp, store) = open().await;
    publish::seed(&store).await;
    let paths = paths(&tmp);
    let mut session = shown(&store, &paths.lock()).await;
    assert!(!press(&mut session, &store, &paths, &[KeyCode::Char('d')]).await);
    let dropped = "SELECT count(*) FROM findings WHERE id = 'f1' AND decision = 'dropped'";
    assert_eq!(queue::count(&store, dropped).await, 1);
}

#[tokio::test]
async fn queued_keep_then_approve_stops_when_reload_replaces_the_draft() {
    let (tmp, store) = open().await;
    publish::seed(&store).await;
    let paths = paths(&tmp);
    let mut session = shown(&store, &paths.lock()).await;
    let head = HEAD;
    store
        .call(move |conn| {
            conn.execute_batch(&format!(
                "INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (3, 'review', 1, 'succeeded', 1, 0, 3, 3);
                 INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (2, 1, 3, 'succeeded', 'no_findings', '{head}', 'base', 'base', 'main', 3);
                 INSERT INTO drafts (id, run_id, original_output, created_at) VALUES (2, 2, '{{}}', 3);"
            ))
        })
        .await
        .unwrap();
    assert_eq!(session.draft_id(), Some(1));
    let keys = [KeyCode::Char('k'), KeyCode::Char('a')];
    assert!(!press(&mut session, &store, &paths, &keys).await);
    assert_eq!(
        queue::count(&store, "SELECT count(*) FROM publications").await,
        0
    );
    assert_eq!(session.notice(), Some("the open draft changed"));
}

#[tokio::test]
async fn queued_keep_then_approve_posts_when_the_draft_stays() {
    let (tmp, store) = open().await;
    publish::seed(&store).await;
    let paths = paths(&tmp);
    let mut session = shown(&store, &paths.lock()).await;
    let keys = [KeyCode::Char('k'), KeyCode::Char('a')];
    assert!(!press(&mut session, &store, &paths, &keys).await);
    assert_eq!(
        queue::count(&store, "SELECT count(*) FROM publications").await,
        1
    );
}

#[tokio::test]
async fn buffered_navigation_cannot_resolve_a_new_confirmation() {
    let (tmp, store) = open().await;
    publish::seed(&store).await;
    let paths = paths(&tmp);
    let mut session = shown(&store, &paths.lock()).await;
    let keys = [KeyCode::Char('x'), KeyCode::Down, KeyCode::Enter];
    assert!(!press(&mut session, &store, &paths, &keys).await);
    assert!(session.confirm_modal().unwrap().highlighted_safe);
    assert_eq!(
        fixture::cell(&store, "SELECT tracking FROM targets").await,
        "tracked"
    );
    session.note_window(80, 8);
    assert!(!press(&mut session, &store, &paths, &[KeyCode::Down]).await);
    assert!(!session.confirm_modal().unwrap().highlighted_safe);
}

#[tokio::test]
async fn queued_approve_needs_a_redraw_after_a_concurrent_edit_to_the_same_draft() {
    let (tmp, store) = open().await;
    publish::seed(&store).await;
    let paths = paths(&tmp);
    let mut session = shown(&store, &paths.lock()).await;
    store
        .save_finding_text("f1", "another title", "another body")
        .await
        .unwrap();
    session.reload(&store).await.unwrap();
    assert_eq!(session.draft_id(), Some(1));
    assert!(!press(&mut session, &store, &paths, &[KeyCode::Char('a')]).await);
    assert_eq!(
        queue::count(&store, "SELECT count(*) FROM publications").await,
        0
    );
    session.note_window(80, 8);
    assert!(!press(&mut session, &store, &paths, &[KeyCode::Char('a')]).await);
    assert_eq!(
        queue::count(&store, "SELECT count(*) FROM publications").await,
        1
    );
}
