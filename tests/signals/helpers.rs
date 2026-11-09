use super::process::{pid, running, wait_until_gone};
use crate::support::git_in;
use crate::support::sandbox::Sandbox;
use crate::tracer::{URL, count, database, with_pr};
use rustix::process::{Signal, kill_process};
use std::process::Stdio;

pub(super) fn interrupted_by(signal: Signal, expected: &str, following: Option<Signal>) {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    sandbox.claude.spawn_child();
    if following.is_some() {
        sandbox.claude.ignore_child_term();
    }
    let hold = sandbox.claude.hold_run(1);
    let mut command = sandbox.diffmate();
    command.args(["review", URL]).stdin(Stdio::null());
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let review = command.spawn().unwrap();
    hold.wait_started();
    let call = sandbox.claude.run_calls().remove(0);
    let (agent, background) = (call.pid.unwrap(), call.child_pid.unwrap());

    kill_process(pid(review.id()), signal).unwrap();
    if let Some(following) = following {
        wait_until_gone(agent);
        kill_process(pid(review.id()), following).unwrap();
    }
    let output = review.wait_with_output().unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("Run cancelled; recorded as interrupted"),
        "{stderr}"
    );
    assert!(!running(agent));
    assert!(!running(background));
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
        (state.as_deref(), kind.as_deref(), cause.as_deref()),
        (Some("failed"), Some("interrupted"), Some(expected))
    );
    assert_eq!((process, group), (agent, agent));
    let (stop, reason): (String, String) = db
        .query_row("SELECT stop_reason, failure_reason FROM jobs", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    assert_eq!(stop, expected);
    assert_eq!(reason, format!("interrupted: {expected}"));
    assert_eq!(
        count(&db, "SELECT count(*) FROM jobs WHERE state = 'failed'"),
        1
    );
    assert!(!sandbox.cache_dir().join("runs/1").exists());
    let repo = sandbox.state_dir().join("repos/o/r.git");
    let listed = git_in(&repo, &sandbox.home, &["worktree", "list", "--porcelain"]);
    assert_eq!(listed.matches("worktree ").count(), 1, "{listed}");
}
