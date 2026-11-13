use super::queue::{self, HEAD, open};
use crate::store::Store;
use ratatui::{backend::TestBackend, crossterm::event::KeyCode};
use std::{os::unix::fs::PermissionsExt, path::Path};
use tempfile::TempDir;
const SCRIPT: &str = "#!/bin/sh\nm=$1\nf=$2\nif [ \"$m\" = \"--touch\" ]; then printf x > \"$2\"; exit 0; fi\nb=$(stat -c %a \"$f\" 2>/dev/null || stat -f %Lp \"$f\")\n[ \"$b\" = 700 ] || exit 4\ncase $m in --fail) printf 'x\\n\\nx\\n' > \"$f\"; exit 1;; --empty) : > \"$f\";; --blank) printf ' \\n' > \"$f\";; --same) ;; --noblank) printf 'Just a title\\n' > \"$f\";; --body) t=$(sed -n 1p \"$f\"); printf '%s\\n\\nfresh body\\n' \"$t\" > \"$f\";; --note) printf 'fresh note' > \"$f\";; *) exit 2;; esac\n";
const FINDING: &str = "INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side, title, body) VALUES ('a', 1, 1, 1, 'high', 'src/a.rs', 4, 'RIGHT', 'One', 'body a');";
const SNAP: &str = "SELECT decision || '|' || body || '|' || title || '|' || original_output || '|' || future_improvements FROM findings, drafts WHERE findings.id = 'a'";
const SAME: &str = "undecided|body a|One|ORIGINAL|[\"keep this\"]";
#[rustfmt::skip]
async fn cell(store: &Store, sql: &str) -> String {
    let sql = sql.to_string();
    store.call(move |conn| conn.query_row(&sql, [], |row| row.get(0))).await.unwrap()
}

#[rustfmt::skip]
async fn draft(findings: bool) -> (TempDir, Store, String) {
    let (tmp, store) = open().await;
    let now = crate::store::now_ms();
    let extra = if findings { FINDING } else { "" };
    let sql = format!("INSERT INTO targets (id, owner, repo, number, tracking, created_at, head_sha, head_checked_at) VALUES (1, 'o', 'r', 1, 'tracked', 1, '{HEAD}', {now}); INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (1, 'review', 1, 'succeeded', 1, 0, 0, 0); INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (1, 1, 1, 'succeeded', 'findings', '{HEAD}', 'base', 'base', 'main', 0); INSERT INTO drafts (id, run_id, original_output, limitation, future_improvements, created_at) VALUES (1, 1, 'ORIGINAL', 'missing CI', '[\"keep this\"]', 0); {extra}");
    store.call(move |conn| conn.execute_batch(&sql)).await.unwrap();
    let path = tmp.path().join("fake-edit");
    std::fs::write(&path, SCRIPT).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    (tmp, store, path.display().to_string())
}

#[rustfmt::skip]
async fn shown(store: &Store, cache: &Path, editor: Option<&str>, keys: &[KeyCode]) -> String {
    let events: Vec<_> = keys.iter().copied().map(queue::key).collect();
    let drawn = crate::tui::run_scripted_with(store, &events, TestBackend::new(140, 12), editor, Some(cache));
    queue::text_of(&drawn.await.unwrap())
}

#[rustfmt::skip]
#[tokio::test(start_paused = true)]
async fn editor_saves_only_a_real_change() {
    let finding = [KeyCode::Enter, KeyCode::Char('e')];
    let note = [KeyCode::Enter, KeyCode::Down, KeyCode::Down, KeyCode::Char('e')];
    let rows = [
        ("--body", false, "edited|fresh body\n|One|ORIGINAL|[\"keep this\"]"),
        ("--fail", false, SAME),
        ("--same", false, SAME),
        ("--empty", false, SAME),
        ("--blank", false, SAME),
        ("--note", true, "undecided|body a|One|ORIGINAL|[\"fresh note\"]"),
        ("--empty", true, SAME),
    ];
    for (flag, on_note, want) in rows {
        let (tmp, store, bin) = draft(true).await;
        let editor = format!("{bin} {flag}");
        let keys: &[KeyCode] = if on_note { &note } else { &finding };
        shown(&store, &tmp.path().join("cache"), Some(&editor), keys).await;
        assert_eq!(cell(&store, SNAP).await, want, "{flag}");
    }
    let (tmp, store, bin) = draft(true).await;
    let cache = tmp.path().join("cache");
    let text = shown(&store, &cache, Some(&format!("{bin} --noblank")), &finding).await;
    assert!(text.contains("the editor file must be a title, a blank line, and a body"), "{text}");
    assert_eq!(cell(&store, SNAP).await, SAME);
    for editor in [None, Some("   ")] {
        let text = shown(&store, &cache, editor, &finding).await;
        assert!(text.contains("EDITOR is not set"), "{text}");
    }
    assert!(cache.read_dir().unwrap().next().is_none());
    let text = shown(&store, &cache, Some("missing-editor-bin"), &finding).await;
    assert!(text.contains("the editor could not be started"), "{text}");
    assert!(cache.read_dir().unwrap().next().is_none());
    assert_eq!(cell(&store, SNAP).await, SAME);
}

#[rustfmt::skip]
#[tokio::test(start_paused = true)]
async fn one_session_names_each_editor_file() {
    let (tmp, store, _) = draft(true).await;
    let log = tmp.path().join("names");
    let bin = tmp.path().join("record");
    let script = format!("#!/bin/sh\nprintf '%s\\n' \"$1\" >> '{}'\n", log.display());
    std::fs::write(&bin, script).unwrap();
    std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o700)).unwrap();
    let cache = tmp.path().join("cache");
    let keys = [KeyCode::Enter, KeyCode::Char('e'), KeyCode::Char('e')];
    shown(&store, &cache, Some(&bin.display().to_string()), &keys).await;
    let names = std::fs::read_to_string(&log).unwrap();
    let lines: Vec<_> = names.lines().collect();
    assert_eq!(lines.len(), 2, "{names}");
    let stem = cache.join(format!("a-{}-", std::process::id()));
    assert_eq!(lines[0], format!("{}0", stem.display()));
    assert_eq!(lines[1], format!("{}1", stem.display()));
    assert!(cache.read_dir().unwrap().next().is_none());
}

#[rustfmt::skip]
#[tokio::test(start_paused = true)]
async fn i_includes_the_limitation_without_starting_an_editor() {
    let (tmp, store, _) = draft(false).await;
    let keys = [KeyCode::Enter, KeyCode::Char('k'), KeyCode::Char('K')];
    let text = shown(&store, &tmp.path().join("cache"), None, &keys).await;
    assert!(text.contains("Limitation: missing CI"), "{text}");
    assert_eq!(cell(&store, "SELECT CAST(count(*) AS TEXT) FROM findings").await, "0");
    let (tmp, store, bin) = draft(true).await;
    let marker = tmp.path().join("ran");
    let touch = format!("{bin} --touch {}", marker.display());
    let cache = tmp.path().join("cache");
    let include = [KeyCode::Enter, KeyCode::Char('i')];
    let text = shown(&store, &cache, Some(&touch), &include).await;
    assert!(text.contains("Limitation: missing CI included"), "{text}");
    assert!(!marker.exists(), "i started the editor");
    let flag = "SELECT CAST(limitation_included AS TEXT) FROM drafts";
    assert_eq!(cell(&store, flag).await, "1");
    let text = shown(&store, &cache, Some(&touch), &include).await;
    assert!(!text.contains(" included"), "{text}");
    assert_eq!(cell(&store, flag).await, "0");
    store.call(|conn| conn.execute("UPDATE drafts SET limitation = NULL", [])).await.unwrap();
    let text = shown(&store, &cache, Some(&touch), &include).await;
    assert!(text.contains("this draft has no limitation"), "{text}");
    assert!(!marker.exists(), "i started the editor");
    assert_eq!(cell(&store, flag).await, "0");
}
