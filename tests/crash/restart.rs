use super::*;

#[test]
fn a_reused_result_survives_a_crash_after_storage() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    assert!(sandbox.run(&["review", URL]).status.success());
    crash_at(sandbox.diffmate().args(["review", URL]), "stored");
    let resumed = sandbox.run(&["review", URL]);
    assert!(resumed.status.success(), "{resumed:?}");
    let db = database(&sandbox);
    assert_eq!(count(&db, "SELECT count(*) FROM runs"), 1);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE state = 'succeeded' AND result_run_id = 1"
        ),
        3
    );
    assert_clean(&sandbox);
}

#[test]
fn restarting_a_crashed_forced_review_keeps_its_force_intent() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    assert!(sandbox.run(&["review", URL]).status.success());
    crash_at(
        sandbox.diffmate().args(["review", URL, "--force"]),
        "agent-running",
    );
    let resumed = sandbox.run(&["review", URL, "--force"]);
    assert!(resumed.status.success(), "{resumed:?}");
    let db = database(&sandbox);
    assert_eq!(count(&db, "SELECT count(*) FROM jobs"), 3);
    assert_eq!(count(&db, "SELECT count(*) FROM runs"), 3);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE id = 3 AND force = 1 AND state = 'succeeded'"
        ),
        1
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM runs WHERE id = 3 AND job_id = 3 AND state = 'succeeded'"
        ),
        1
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM targets WHERE tracking = 'untracked'"
        ),
        1
    );
    assert_clean(&sandbox);
}
