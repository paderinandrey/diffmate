mod support;
#[allow(dead_code)]
mod tracer;

use rustix::process::{Pid, Signal, kill_process};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use support::git_in;
use support::sandbox::Sandbox;
use tracer::{URL, count, database, with_pr};

fn pid(raw: u32) -> Pid {
    Pid::from_raw(i32::try_from(raw).unwrap()).unwrap()
}

fn running(raw: u32) -> bool {
    let mut ps = Command::new("ps");
    let output = ps
        .args(["-o", "stat=", "-p", &raw.to_string()])
        .output()
        .unwrap();
    let stat = String::from_utf8_lossy(&output.stdout);
    !stat.trim().is_empty() && !stat.trim().starts_with('Z')
}

fn wait_gone(raw: u32) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while running(raw) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(!running(raw), "process {raw} is still running");
}

fn interrupted_by(signal: Signal) {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    sandbox.claude.spawn_child();
    let hold = sandbox.claude.hold_run(1);
    let mut command = sandbox.diffmate();
    command.args(["review", URL]).stdin(Stdio::null());
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let review = command.spawn().unwrap();
    hold.wait_started();
    let call = sandbox.claude.run_calls().remove(0);
    let (agent, background) = (call.pid.unwrap(), call.child_pid.unwrap());

    kill_process(pid(review.id()), signal).unwrap();
    let output = review.wait_with_output().unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("Run cancelled; recorded as interrupted"),
        "{stderr}"
    );
    assert!(!running(agent));
    wait_gone(background);
    let db = database(&sandbox);
    let sql = "SELECT state, failure_kind, interrupted_cause, agent_pid, agent_pgid FROM runs";
    let row = |r: &rusqlite::Row| {
        let text = |n| r.get::<_, Option<String>>(n);
        Ok((
            text(0)?,
            text(1)?,
            text(2)?,
            r.get::<_, u32>(3)?,
            r.get::<_, u32>(4)?,
        ))
    };
    let (state, kind, cause, process, group) = db.query_row(sql, [], row).unwrap();
    assert_eq!(
        (state.as_deref(), kind.as_deref(), cause),
        (Some("failed"), Some("interrupted"), None)
    );
    assert_eq!((process, group), (agent, agent));
    assert_eq!(
        count(&db, "SELECT count(*) FROM jobs WHERE state = 'failed'"),
        1
    );
    assert!(!sandbox.cache_dir().join("runs/1").exists());
    let repo = sandbox.state_dir().join("repos/o/r.git");
    let listed = git_in(&repo, &sandbox.home, &["worktree", "list", "--porcelain"]);
    assert_eq!(listed.matches("worktree ").count(), 1, "{listed}");
}

#[test]
fn sigint_during_agent_run_stops_group_and_records_interrupted() {
    interrupted_by(Signal::INT);
}

#[test]
fn sigterm_during_agent_run_is_handled_the_same_way() {
    interrupted_by(Signal::TERM);
}
