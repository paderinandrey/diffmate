use super::*;
use crate::support::git_in;

fn stopped_automatic(signal: Signal, repeated: bool) {
    let sandbox = Sandbox::new();
    concurrency::with_two_prs(&sandbox);
    sandbox.claude.spawn_child();
    if repeated {
        sandbox.claude.ignore_child_term();
    }
    assert!(
        debug_enqueue(&sandbox, URL, &["--automatic"])
            .status
            .success()
    );
    assert!(
        debug_enqueue(&sandbox, &concurrency::url(2), &[])
            .status
            .success()
    );
    let hold = sandbox.claude.hold_run(1);
    let child = spawn_watch(&sandbox);
    hold.wait_started();
    let call = sandbox.claude.run_calls().remove(0);
    let (agent, background) = (call.pid.unwrap(), call.child_pid.unwrap());
    kill_process(process::pid(child.id()), signal).unwrap();
    if repeated {
        process::wait_until_gone(agent);
        kill_process(process::pid(child.id()), Signal::TERM).unwrap();
    }
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, [] as [u8; 0]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("job 1 review o/r#1 failed (interrupted: shutdown), requeued\n"),
        "{stderr}"
    );
    assert_eq!(stderr.matches(", requeued").count(), 1);
    assert!(stderr.contains("watch stopped (shutdown)"));
    let db = database(&sandbox);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM runs WHERE state = 'failed' AND interrupted_cause = 'shutdown'"
        ),
        1
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE id = 1 AND state = 'queued' AND retry_count = 0 AND next_attempt_at IS NULL AND stop_reason IS NULL"
        ),
        1
    );
    assert_eq!(
        count(&db, "SELECT count(*) FROM runs WHERE target_id = 2"),
        0
    );
    assert!(!sandbox.cache_dir().join("runs/1").exists());
    let repo = sandbox.state_dir().join("repos/o/r.git");
    let listed = git_in(&repo, &sandbox.home, &["worktree", "list", "--porcelain"]);
    assert_eq!(listed.matches("worktree ").count(), 1);
    process::wait_until_gone(agent);
    process::wait_until_gone(background);
    let mut restarted = spawn_watch(&sandbox);
    let mut reader = BufReader::new(restarted.stderr.take().unwrap());
    read_until(&mut reader, "job 1 review o/r#1 succeeded");
    kill_process(process::pid(restarted.id()), Signal::TERM).unwrap();
    assert!(restarted.wait().unwrap().success());
}

#[test]
fn sigterm_during_an_automatic_run_requeues_it() {
    stopped_automatic(Signal::TERM, false);
}

#[test]
fn sigint_during_an_automatic_run_is_shutdown() {
    stopped_automatic(Signal::INT, false);
}

#[test]
fn a_second_sigterm_during_the_stop_is_ignored() {
    stopped_automatic(Signal::TERM, true);
}
