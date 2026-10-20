#[path = "concurrency/helpers.rs"]
mod helpers;
mod support;
#[allow(dead_code)]
mod tracer;

use helpers::{spawn_review, with_two_prs};
use support::git_in;
use support::sandbox::Sandbox;
use tracer::{count, database};

#[test]
fn killed_runner_is_recovered_by_next_runner() {
    let sandbox = Sandbox::new();
    with_two_prs(&sandbox);
    let hold = sandbox.claude.hold_run(1);
    let mut first = spawn_review(&sandbox, 1);
    hold.wait_started();
    assert!(sandbox.cache_dir().join("runs/1/worktree").exists());
    first.kill().unwrap();
    first.wait().unwrap();
    hold.release();

    let second = spawn_review(&sandbox, 2).wait_with_output().unwrap();

    let stderr = String::from_utf8_lossy(&second.stderr);
    assert!(second.status.success(), "{stderr}");
    let marked = "recovery: run 1 marked failed (interrupted: crash)";
    assert!(stderr.lines().any(|line| line == marked), "{stderr}");
    let db = database(&sandbox);
    let sql = "SELECT state, failure_kind, interrupted_cause FROM runs WHERE id = 1";
    let row = |r: &rusqlite::Row| Ok((r.get::<_, String>(0)?, r.get(1)?, r.get::<_, String>(2)?));
    let (state, kind, cause): (String, String, String) = db.query_row(sql, [], row).unwrap();
    assert_eq!(
        (state.as_str(), kind.as_str(), cause.as_str()),
        ("failed", "interrupted", "crash")
    );
    let sql = "SELECT state FROM jobs WHERE id = 1";
    assert_eq!(
        db.query_row(sql, [], |r| r.get::<_, String>(0)).unwrap(),
        "failed"
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM runs WHERE id = 2 AND state = 'succeeded'"
        ),
        1
    );
    assert!(!sandbox.cache_dir().join("runs/1").exists());
    let repo = sandbox.state_dir().join("repos/o/r.git");
    let listed = git_in(&repo, &sandbox.home, &["worktree", "list", "--porcelain"]);
    assert_eq!(listed.matches("worktree ").count(), 1, "{listed}");
    let status = String::from_utf8(sandbox.run(&["status"]).stdout).unwrap();
    assert!(
        status.starts_with("o/r#1  untracked  failed (interrupted: crash)\n"),
        "{status}"
    );
}
