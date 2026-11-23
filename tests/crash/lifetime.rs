use super::{Sandbox, URL, assert_clean, process, watch_helpers, with_pr};
use rustix::process::{Pid, test_kill_process_group};
use std::os::unix::process::CommandExt;
use std::process::Command;

#[test]
fn a_crash_before_launch_authorization_never_starts_the_agent() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    super::crash_at(sandbox.diffmate().args(["review", URL]), "agent-guarded");
    let resumed = sandbox.run(&["review", URL]);
    assert!(resumed.status.success(), "{resumed:?}");
    assert_eq!(sandbox.claude.run_calls().len(), 1);
    assert_clean(&sandbox);
}

#[test]
fn recovery_waits_for_descendants_even_before_the_spawn_is_recorded() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    sandbox.claude.spawn_child();
    sandbox.claude.ignore_child_term();
    let hold = sandbox.claude.hold_run(1);
    let (mut first, mut log) =
        process::spawn_at(sandbox.diffmate().args(["review", URL]), "agent-spawned");
    watch_helpers::read_until(&mut log, "crash point reached: agent-spawned");
    hold.wait_started();
    let call = sandbox.claude.run_calls().remove(0);
    let mut unrelated = process::HeldChild(
        Command::new("sleep")
            .arg("300")
            .process_group(0)
            .spawn()
            .unwrap(),
    );
    let db = rusqlite::Connection::open(sandbox.db_path()).unwrap();
    assert_eq!(
        db.execute(
            "UPDATE runs SET agent_pid = ?1, agent_pgid = ?1 WHERE id = 1 AND agent_pid IS NULL",
            [unrelated.0.id()],
        )
        .unwrap(),
        1
    );
    process::kill(&mut first.0);

    let (mut next, mut log) =
        process::spawn_at(sandbox.diffmate().args(["review", URL]), "recovery-db");
    watch_helpers::read_until(&mut log, "crash point reached: recovery-db");
    let pgid = Pid::from_raw(call.pid.unwrap().try_into().unwrap()).unwrap();
    assert!(call.child_pid.is_some());
    assert!(test_kill_process_group(pgid).is_err());
    assert!(unrelated.0.try_wait().unwrap().is_none());
    assert!(sandbox.cache_dir().join("runs/1/worktree").exists());
    process::kill(&mut next.0);
}

#[test]
fn a_successful_agent_that_leaves_a_background_child_cannot_outlive_its_worktree() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    sandbox.claude.spawn_child();
    sandbox.claude.ignore_child_term();
    let result = sandbox.run(&["review", URL]);
    assert!(result.status.success(), "{result:?}");
    let call = sandbox.claude.run_calls().remove(0);
    let pgid = Pid::from_raw(call.pid.unwrap().try_into().unwrap()).unwrap();
    assert!(call.child_pid.is_some());
    assert!(test_kill_process_group(pgid).is_err());
    assert_clean(&sandbox);
}
