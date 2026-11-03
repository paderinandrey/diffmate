#[path = "crash/helpers.rs"]
mod helpers;
#[path = "crash/matrix.rs"]
mod matrix;
#[path = "crash/restart.rs"]
mod restart;
mod support;
#[allow(dead_code)]
mod tracer;

use helpers::{assert_clean, crash_at};
use support::sandbox::Sandbox;
use tracer::{URL, count, database, with_pr};

#[test]
fn killed_while_the_agent_runs_leaves_no_duplicate_and_no_worktree() {
    let sandbox = Sandbox::new();
    let (_, head) = with_pr(&sandbox);
    crash_at(sandbox.diffmate().args(["review", URL]), "agent-running");
    let resumed = sandbox.run(&["review", URL]);
    assert!(resumed.status.success(), "{resumed:?}");
    let db = database(&sandbox);
    assert_eq!(count(&db, "SELECT count(*) FROM jobs"), 2);
    assert_eq!(count(&db, "SELECT count(*) FROM runs"), 2);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE id = 1 AND state = 'failed'"
        ),
        1
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM runs WHERE id = 1 AND job_id = 1 AND state = 'failed' AND failure_kind = 'interrupted' AND interrupted_cause = 'crash'"
        ),
        1
    );
    assert_eq!(
        db.query_row(
            "SELECT head_sha FROM runs WHERE id = 2 AND job_id = 2 AND state = 'succeeded'",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        head
    );
    assert_eq!(
        count(&db, "SELECT count(*) FROM runs WHERE state = 'succeeded'"),
        1
    );
    assert_clean(&sandbox);
}
