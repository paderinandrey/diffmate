use super::fake::Fake;
use super::queue::{self, HEAD, key, open};
use crate::paths::Paths;
use crate::store::Store;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;

pub(super) async fn seed(store: &Store) {
    let now = crate::store::now_ms();
    let sql = format!(
        "INSERT INTO targets (id, owner, repo, number, tracking, created_at, head_sha, head_checked_at) VALUES (1, 'o', 'r', 1, 'tracked', 1, '{HEAD}', {now});
         INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (1, 'review', 1, 'succeeded', 1, 0, 0, 0);
         INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (1, 1, 1, 'succeeded', 'findings', '{HEAD}', 'base', 'base', 'main', 0);
         INSERT INTO drafts (id, run_id, original_output, created_at) VALUES (1, 1, '{{}}', 0);
         INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body, decision) VALUES ('f1', 1, 1, 1, 'high', 'src/a.rs', 4, 'RIGHT', 'Title', 'Body', 'kept');
         INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (2, 'review', 1, 'queued', 1, 0, 1, 1);"
    );
    store
        .call(move |conn| conn.execute_batch(&sql))
        .await
        .unwrap();
}

pub(super) async fn press(store: &Store, paths: &Paths, gh: &Fake, keys: &[KeyCode]) -> String {
    let git = crate::git::GitCli::new(&crate::procenv::InheritedEnv::from_pairs(Vec::new()));
    let settings = crate::config::Settings::default();
    let events: Vec<_> = keys.iter().copied().map(key).collect();
    let drawn = crate::tui::run_scripted_publish(
        store,
        &events,
        TestBackend::new(160, 8),
        paths,
        gh,
        &git,
        &settings,
    )
    .await
    .unwrap();
    super::queue::text_of(&drawn)
}

#[tokio::test]
async fn comment_key_posts_one_review() {
    let (tmp, store) = open().await;
    seed(&store).await;
    let gh = Fake::new(HEAD);
    let paths = Paths::new(tmp.path().join("state"), tmp.path().join("cache"));
    let text = press(
        &store,
        &paths,
        &gh,
        &[KeyCode::Enter, KeyCode::Char('c'), KeyCode::Esc],
    )
    .await;
    assert!(text.contains("runner: tui"), "{text}");
    let seen = gh.seen();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].event, "COMMENT");
    let submitted = "SELECT count(*) FROM publications WHERE state = 'submitted'";
    assert_eq!(queue::count(&store, submitted).await, 1);
    let queued = "SELECT count(*) FROM jobs WHERE id = 2 AND state = 'queued'";
    assert_eq!(queue::count(&store, queued).await, 1);
}

pub(super) fn paths(tmp: &tempfile::TempDir) -> Paths {
    Paths::new(tmp.path().join("state"), tmp.path().join("cache"))
}

pub(super) async fn batch(store: &Store, sql: String) {
    store
        .call(move |conn| conn.execute_batch(&sql))
        .await
        .unwrap();
}

pub(super) fn base(now: i64, findings: &str) -> String {
    format!(
        "INSERT INTO targets (id, owner, repo, number, tracking, created_at, head_sha, head_checked_at) VALUES (1, 'o', 'r', 1, 'tracked', 1, '{HEAD}', {now});
         INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (1, 'review', 1, 'succeeded', 1, 0, 0, 0);
         INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (1, 1, 1, 'succeeded', 'findings', '{HEAD}', 'base', 'base', 'main', 0);
         INSERT INTO drafts (id, run_id, original_output, created_at) VALUES (1, 1, '{{}}', 0);
         {findings}"
    )
}

pub(super) fn hold(path: &std::path::Path) -> std::fs::File {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)
        .unwrap();
    file.try_lock().unwrap();
    file
}
