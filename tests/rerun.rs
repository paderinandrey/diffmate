#[path = "rerun/cancel.rs"]
mod cancel;
#[path = "rerun/delayed.rs"]
mod delayed;
#[allow(dead_code)]
#[path = "watch/helpers.rs"]
mod helpers;
#[allow(dead_code)]
#[path = "concurrency/helpers.rs"]
mod processes;
mod support;
#[allow(dead_code)]
mod tracer;

use helpers::debug_enqueue;
use support::sandbox::Sandbox;
use tracer::{URL, count, database};

#[test]
fn review_runs_only_the_rerun_it_attached_to() {
    let sandbox = Sandbox::new();
    processes::with_two_prs(&sandbox);
    for (number, flags) in [(1, vec![]), (2, vec!["--automatic"]), (3, vec![])] {
        assert!(
            debug_enqueue(&sandbox, &processes::url(number), &flags)
                .status
                .success()
        );
    }
    let output = sandbox.run(&["review", URL]);
    assert!(output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("attached to job 1"));
    let db = database(&sandbox);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE id = 1 AND state = 'succeeded'"
        ),
        1
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE id IN (2, 3) AND state = 'queued'"
        ),
        2
    );
    assert_eq!(
        count(&db, "SELECT count(*) FROM runs WHERE target_id IN (2, 3)"),
        0
    );
    assert_eq!(sandbox.claude.run_calls().len(), 1);
}

#[test]
fn forced_rerun_makes_a_new_run() {
    let sandbox = Sandbox::new();
    tracer::with_pr(&sandbox);
    assert!(sandbox.run(&["review", URL]).status.success());
    assert!(debug_enqueue(&sandbox, URL, &["--force"]).status.success());
    let output = sandbox.run(&["review", URL]);
    assert!(output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("attached to job 2"));
    assert_eq!(count(&database(&sandbox), "SELECT count(*) FROM runs"), 2);
}
