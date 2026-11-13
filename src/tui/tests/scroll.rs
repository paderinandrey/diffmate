use super::queue::{self, key, open};
use crate::tui::run_scripted;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;

async fn shown(
    store: &crate::store::Store,
    downs: usize,
    extra: &[KeyCode],
) -> (String, Vec<String>) {
    let mut keys = vec![key(KeyCode::Enter)];
    keys.extend(std::iter::repeat_n(key(KeyCode::Down), downs));
    keys.extend(extra.iter().copied().map(key));
    let drawn = run_scripted(store, &keys, TestBackend::new(10, 6))
        .await
        .unwrap();
    let text = queue::text_of(&drawn);
    let rows = text
        .lines()
        .map(|line| line.trim_matches('"').trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();
    (text, rows)
}

#[tokio::test]
async fn down_scrolls_a_finding_taller_than_the_screen() {
    let (_tmp, store) = open().await;
    let now = crate::store::now_ms();
    let body = format!("{}TAIL", "a".repeat(400));
    let head = queue::HEAD;
    let sql = format!(
        "INSERT INTO targets (id, owner, repo, number, tracking, created_at, head_sha, head_checked_at) VALUES (1, 'o', 'r', 1, 'tracked', 1, '{head}', {now});
         INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (1, 'review', 1, 'succeeded', 1, 0, 0, 0);
         INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (1, 1, 1, 'succeeded', 'findings', '{head}', 'base', 'base', 'main', 0);
         INSERT INTO drafts (id, run_id, original_output, created_at) VALUES (1, 1, '{{}}', 0);
         INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body) VALUES ('long', 1, 1, 1, 'high', 'a.rs', 1, 'RIGHT', 'Long', '{body}');
         INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body) VALUES ('next', 1, 1, 2, 'low', 'a.rs', 2, 'RIGHT', 'Next', 'short');"
    );
    store
        .call(move |conn| conn.execute_batch(&sql))
        .await
        .unwrap();
    let mut tail_at = None;
    let mut next_at = None;
    for downs in 0..80 {
        let (text, _) = shown(&store, downs, &[]).await;
        if tail_at.is_none() && text.contains("TAIL") {
            assert!(!text.contains("Next"), "{text}");
            tail_at = Some(downs);
        }
        if text.contains("Next") {
            next_at = Some(downs);
            break;
        }
    }
    let tail_at = tail_at.expect("tail stayed hidden");
    let next_at = next_at.expect("never reached the next finding");
    assert!(tail_at > 0, "tail was visible before scrolling");
    assert!(next_at > tail_at);
    let (back, _) = shown(&store, next_at, &[KeyCode::Up]).await;
    assert!(!back.contains("TAIL"), "{back}");
    assert!(back.contains("Long"), "{back}");
}
