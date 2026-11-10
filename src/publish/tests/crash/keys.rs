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
