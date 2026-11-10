use super::support::{batch, cell, open};
use crate::paths::Paths;
use crate::tui::Fake;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

const RUN: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const LIVE: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

async fn seed(store: &crate::store::Store) {
    let now = crate::store::now_ms();
    let sql = format!(
        "INSERT INTO targets (id, owner, repo, number, tracking, created_at, head_sha, head_checked_at) VALUES (1, 'o', 'r', 1, 'tracked', 1, '{RUN}', {now});
         INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (1, 'review', 1, 'succeeded', 1, 0, 0, 0);
         INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (1, 1, 1, 'succeeded', 'findings', '{RUN}', 'base', 'base', 'main', 0);
         INSERT INTO drafts (id, run_id, original_output, created_at) VALUES (1, 1, '{{}}', 0);
         INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body, decision) VALUES ('f1', 1, 1, 1, 'high', 'src/a.rs', 4, 'RIGHT', 'Title', 'Body', 'kept');"
    );
    batch(store, &sql).await;
}

async fn press(store: &crate::store::Store, paths: &Paths, gh: &Fake, keys: &[KeyCode]) -> String {
    let git = crate::git::GitCli::new(&crate::procenv::InheritedEnv::from_pairs(Vec::new()));
    let settings = crate::config::Settings::default();
    let events: Vec<_> = keys.iter().copied().map(key).collect();
    let drawn = crate::tui::run_scripted_publish(
        store,
        &events,
        ratatui::backend::TestBackend::new(120, 8),
        paths,
        gh,
        &git,
        &settings,
    )
    .await
    .unwrap();
    drawn.to_string()
}

#[tokio::test]
async fn stale_highlight_does_not_send() {
    let (tmp, store) = open().await;
    seed(&store).await;
    let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
    let gh = Fake::new(LIVE);
    let text = press(&store, &paths, &gh, &[KeyCode::Enter, KeyCode::Char('c')]).await;
    assert!(text.contains("[Don't send]"), "{text}");
    assert!(!text.contains("[Send anyway]"), "{text}");
    assert!(
        text.contains("stale: head bbbbbbbbbbbb, checked just now"),
        "{text}"
    );
    press(
        &store,
        &paths,
        &gh,
        &[KeyCode::Enter, KeyCode::Char('c'), KeyCode::Enter],
    )
    .await;
    assert_eq!(
        cell::<i64>(&store, "SELECT count(*) FROM publications").await,
        0
    );
    assert_eq!(
        cell::<i64>(&store, "SELECT count(*) FROM jobs WHERE kind = 'publish'").await,
        0
    );
    assert_eq!(gh.seen(), Vec::new());
    assert_eq!(
        cell::<String>(&store, "SELECT head_sha FROM targets").await,
        LIVE
    );
    press(
        &store,
        &paths,
        &gh,
        &[
            KeyCode::Enter,
            KeyCode::Char('c'),
            KeyCode::Right,
            KeyCode::Enter,
        ],
    )
    .await;
    assert_eq!(
        cell::<String>(&store, "SELECT confirmed_sha FROM publications").await,
        LIVE
    );
    let seen = gh.seen();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].commit_id, RUN);
}
