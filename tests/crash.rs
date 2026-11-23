#[path = "crash/budget.rs"]
mod budget;
#[allow(dead_code)]
#[path = "concurrency/helpers.rs"]
mod concurrency;
#[path = "crash/helpers.rs"]
mod helpers;
#[path = "crash/lifetime.rs"]
mod lifetime;
#[path = "crash/manual_stop.rs"]
mod manual_stop;
#[path = "crash/matrix.rs"]
mod matrix;
#[path = "crash/process.rs"]
mod process;
#[path = "crash/restart.rs"]
mod restart;
#[path = "crash/stop_stored.rs"]
mod stop_stored;
mod support;
#[allow(dead_code)]
mod tracer;
#[path = "watch/helpers.rs"]
mod watch_helpers;

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

#[test]
fn recovery_drains_a_version_probe_before_marking_the_run_interrupted() {
    use rustix::process::{Pid, test_kill_process_group};
    use std::fs;

    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    let version = sandbox.claude.dir.join("version");
    fs::remove_file(&version).unwrap();
    support::mkfifo(&version);
    let mut first = process::HeldChild(sandbox.diffmate().args(["review", URL]).spawn().unwrap());
    let _blocked_probe = fs::OpenOptions::new().write(true).open(&version).unwrap();
    let call = sandbox.claude.calls().remove(0);
    assert_eq!(call.argv, ["--version"]);
    let pgid = Pid::from_raw(call.pid.unwrap().try_into().unwrap()).unwrap();
    process::kill(&mut first.0);

    let (mut next, mut log) =
        process::spawn_at(sandbox.diffmate().args(["review", URL]), "recovery-db");
    watch_helpers::read_until(&mut log, "crash point reached: recovery-db");
    assert!(test_kill_process_group(pgid).is_err());
    process::kill(&mut next.0);
}
