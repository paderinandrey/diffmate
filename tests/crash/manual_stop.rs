use super::*;
use rustix::process::Signal;

fn stored_stop_is_terminal(signal: Signal, cause: &str) {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    process::crash_after_stop(&sandbox, sandbox.diffmate().args(["review", URL]), signal);
    let resumed = sandbox.run(&["review", URL]);
    assert_eq!(resumed.status.code(), Some(1), "{resumed:?}");
    let text = String::from_utf8_lossy(&resumed.stderr);
    assert!(
        text.contains(&format!("job 1 failed (interrupted: {cause})")),
        "{text}"
    );
    let db = database(&sandbox);
    assert_eq!(count(&db, "SELECT count(*) FROM jobs"), 1);
    assert_eq!(count(&db, "SELECT count(*) FROM runs"), 1);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE id = 1 AND origin = 'manual' AND state = 'failed' AND retry_count = 0"
        ),
        1
    );
    assert_eq!(
        db.query_row(
            "SELECT interrupted_cause FROM runs WHERE id = 1 AND state = 'failed'",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        cause
    );
    assert_clean(&sandbox);
}

#[test]
fn a_manual_shutdown_recovered_by_review_does_not_create_another_job() {
    stored_stop_is_terminal(Signal::TERM, "shutdown");
}

#[test]
fn a_manual_user_cancel_recovered_by_review_does_not_create_another_job() {
    stored_stop_is_terminal(Signal::INT, "user_cancel");
}
