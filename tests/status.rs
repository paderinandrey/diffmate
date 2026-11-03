#[path = "status/helpers.rs"]
mod helpers;
mod support;
#[allow(dead_code)]
mod tracer;

use helpers::{one_finding, snapshot, status_lines};
use support::fake_claude::{output_json, success_envelope};
use support::sandbox::Sandbox;
use tracer::{count, database, review, with_pr};

const SECOND: &str = "https://github.com/o/r/pull/2";

#[test]
fn status_without_database_prints_no_targets_and_creates_nothing() {
    let sandbox = Sandbox::new();

    let output = sandbox.run(&["status"]);

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "No targets.\n");
    assert!(!sandbox.state_dir().exists());
}

#[test]
fn status_lists_latest_run_per_target() {
    let sandbox = Sandbox::new();
    let (_, head) = with_pr(&sandbox);
    sandbox.claude.set_run(&one_finding("Off by one"), 0);
    assert!(review(&sandbox).status.success());
    let second = sandbox.run(&["review", SECOND]);
    assert_eq!(second.status.code(), Some(1));

    let calls = sandbox.gh.calls().len();
    let lines = status_lines(&sandbox);

    assert_eq!(lines.len(), 2, "{lines:?}");
    assert_eq!(sandbox.gh.calls().len(), calls + 1);
    assert_eq!(
        lines[0],
        format!(
            "o/r#1  untracked  succeeded (findings: 1)  current: head {}, checked just now",
            &head[..12]
        )
    );
    let prefix = "o/r#2  untracked  failed before run: ";
    assert!(lines[1].starts_with(prefix), "{}", lines[1]);
}

#[test]
fn two_reviews_of_one_url_create_two_runs() {
    let sandbox = Sandbox::new();
    let (_, head) = with_pr(&sandbox);
    sandbox.claude.set_run(&one_finding("First"), 0);
    assert!(review(&sandbox).status.success());
    sandbox
        .claude
        .set_run(&success_envelope(&output_json(&[], None, &[])), 0);
    assert!(review(&sandbox).status.success());

    let db = database(&sandbox);
    assert_eq!(count(&db, "SELECT count(*) FROM jobs"), 2);
    assert_eq!(count(&db, "SELECT count(*) FROM runs"), 2);
    assert_eq!(count(&db, "SELECT count(*) FROM drafts"), 2);
    let sql = "SELECT title FROM findings WHERE run_id = 1";
    let title: String = db.query_row(sql, [], |r| r.get(0)).unwrap();
    assert_eq!(title, "First");
    assert_eq!(
        count(&db, "SELECT count(*) FROM findings WHERE run_id = 2"),
        0
    );
    let lines = status_lines(&sandbox);
    assert_eq!(
        lines,
        [format!(
            "o/r#1  untracked  succeeded (no_findings)  current: head {}, checked just now",
            &head[..12]
        )]
    );
}

#[test]
fn status_does_not_write() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    assert!(review(&sandbox).status.success());
    let before = snapshot(&database(&sandbox));

    let lines = status_lines(&sandbox);

    assert_eq!(lines.len(), 1);
    assert_eq!(snapshot(&database(&sandbox)), before);
}
