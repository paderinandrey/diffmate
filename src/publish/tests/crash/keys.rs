use super::gh::Remote;
use super::harness::{self, PUBLICATION, planted};
use crate::gh::ListedReview;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn review(body: &str) -> ListedReview {
    ListedReview {
        id: "77".into(),
        author: "tester".into(),
        state: "COMMENTED".into(),
        body: body.into(),
    }
}

async fn shown(
    store: &crate::store::Store,
    tmp: &tempfile::TempDir,
    remote: &Remote,
    keys: &[KeyCode],
) -> String {
    let events: Vec<_> = keys.iter().copied().map(key).collect();
    let git = crate::git::GitCli::new(&crate::procenv::InheritedEnv::from_pairs(Vec::new()));
    let drawn = crate::tui::run_scripted_publish(
        store,
        &events,
        TestBackend::new(160, 8),
        &harness::paths(tmp),
        remote,
        &git,
        &crate::config::Settings::default(),
    )
    .await
    .unwrap();
    drawn.to_string()
}

#[tokio::test]
async fn y_marker_draws_submitted() {
    let (tmp, store, remote) = planted("unknown").await;
    let marker = format!("<!-- diffmate:publication={PUBLICATION} -->");
    remote.reviews(vec![review(&marker)]);
    let text = shown(&store, &tmp, &remote, &[KeyCode::Char('y')]).await;
    assert!(text.contains("submitted"), "{text}");
    assert_eq!(remote.posts(), 0);
}

#[tokio::test]
async fn y_missing_marker_stays_unknown() {
    let (tmp, store, remote) = planted("unknown").await;
    remote.reviews(vec![review("no marker")]);
    let text = shown(&store, &tmp, &remote, &[KeyCode::Char('y')]).await;
    assert!(text.contains("unknown"), "{text}");
    assert_eq!(
        harness::text(&store, "SELECT state FROM publications").await,
        "unknown"
    );
    assert_eq!(
        harness::text(&store, "SELECT state FROM jobs WHERE kind = 'reconcile'").await,
        "succeeded"
    );
    assert_eq!(remote.posts(), 0);
}

#[tokio::test]
async fn y_when_not_unknown_does_nothing() {
    let (tmp, store, remote) = planted("submitted").await;
    shown(&store, &tmp, &remote, &[KeyCode::Char('y')]).await;
    let sql = "SELECT CASE WHEN EXISTS (SELECT 1 FROM jobs WHERE kind = 'reconcile') THEN 'yes' ELSE 'no' END";
    assert_eq!(harness::text(&store, sql).await, "no");
    assert_eq!(
        harness::text(&store, "SELECT state FROM publications").await,
        "submitted"
    );
    assert_eq!(remote.posts(), 0);
}

#[tokio::test]
async fn y_on_the_open_draft_submits() {
    let (tmp, store, remote) = planted("unknown").await;
    let marker = format!("<!-- diffmate:publication={PUBLICATION} -->");
    remote.reviews(vec![review(&marker)]);
    shown(
        &store,
        &tmp,
        &remote,
        &[KeyCode::Enter, KeyCode::Char('y'), KeyCode::Esc],
    )
    .await;
    assert_eq!(
        harness::text(&store, "SELECT state FROM publications").await,
        "submitted"
    );
    assert_eq!(remote.posts(), 0);
}

#[tokio::test]
async fn y_runs_reconcile_after_a_due_sync() {
    let (tmp, store, remote) = planted("unknown").await;
    let synced = "fedcba9876543210fedcba9876543210";
    let review_id = "5150";
    let sql = format!(
        "INSERT INTO publications (id, target_id, draft_id, run_id, state, event, confirmed_sha, snapshot, github_review_id, created_at, updated_at) VALUES ('{synced}', 1, 1, 1, 'submitted', 'COMMENT', 'abc123', '{{}}', '{review_id}', 1, 1);"
    );
    super::super::support::batch(&store, &sql).await;
    store.enqueue_sync(synced).await.unwrap();
    shown(&store, &tmp, &remote, &[KeyCode::Char('y')]).await;
    assert_eq!(remote.posts(), 0);
    let publication = format!("SELECT state FROM publications WHERE id = '{PUBLICATION}'");
    assert_eq!(harness::text(&store, &publication).await, "unknown");
    assert_ne!(harness::text(&store, &publication).await, "rejected");
    assert_eq!(
        harness::text(&store, "SELECT state FROM jobs WHERE kind = 'reconcile'").await,
        "succeeded"
    );
    let submitted = format!("SELECT state FROM publications WHERE id = '{synced}'");
    assert_eq!(harness::text(&store, &submitted).await, "submitted");
    let stored_review = format!("SELECT github_review_id FROM publications WHERE id = '{synced}'");
    assert_eq!(harness::text(&store, &stored_review).await, review_id);
    assert_eq!(
        harness::text(&store, "SELECT thread_status FROM findings WHERE id = 'f1'").await,
        "unknown"
    );
    let sync_job =
        format!("SELECT state FROM jobs WHERE kind = 'sync' AND publication_id = '{synced}'");
    assert_eq!(harness::text(&store, &sync_job).await, "queued");
    let due = format!(
        "SELECT next_attempt_at IS NOT NULL FROM jobs WHERE kind = 'sync' AND publication_id = '{synced}'"
    );
    assert_eq!(super::super::support::cell::<i64>(&store, &due).await, 1);
}
