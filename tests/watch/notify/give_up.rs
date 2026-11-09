use super::super::super::{URL, debug_enqueue, process, with_pr};
use super::super::config;
use super::support;
use crate::support::sandbox::Sandbox;
use rustix::process::{Signal, kill_process};

const REASON: &str = "quota-marker-zz";

fn envelope(reason: &str) -> String {
    format!(
        "{{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":true,\"api_error_status\":429,\"duration_ms\":1,\"duration_api_ms\":1,\"num_turns\":1,\"result\":\"{reason}\",\"total_cost_usd\":0,\"session_id\":\"00000000-0000-0000-0000-000000000000\"}}"
    )
}

#[test]
fn an_automatic_give_up_names_the_reason() {
    let sandbox = Sandbox::new();
    config(&sandbox, "max_retries = 0\n");
    let script = support::install(&sandbox);
    with_pr(&sandbox);
    sandbox.claude.set_run(&envelope(REASON), 1);
    assert!(
        debug_enqueue(&sandbox, URL, &["--automatic"])
            .status
            .success()
    );
    let mut child = support::spawn(&sandbox);
    let stderr = support::finish(&mut child, "job 1 review o/r#1 failed");
    let log = support::text(&script);
    let scripts = support::scripts(&log);
    assert_eq!(scripts.len(), 1, "{stderr}");
    assert!(
        scripts[0].contains("diffmate: review failed"),
        "{scripts:?}"
    );
    assert!(scripts[0].contains("o/r#1"), "{scripts:?}");
    assert!(scripts[0].contains(REASON), "{scripts:?}");
    assert!(!stderr.contains("notification-failed"), "{stderr}");
    assert_eq!(
        support::count_sql(&sandbox, "SELECT count(*) FROM jobs WHERE state = 'failed'"),
        1
    );
    assert_eq!(
        support::count_sql(&sandbox, "SELECT count(*) FROM jobs WHERE state = 'queued'"),
        0
    );
}

#[test]
fn sigterm_during_an_automatic_run_does_not_notify() {
    let sandbox = Sandbox::new();
    let script = support::install(&sandbox);
    with_pr(&sandbox);
    sandbox.claude.spawn_child();
    let hold = sandbox.claude.hold_run(1);
    assert!(
        debug_enqueue(&sandbox, URL, &["--automatic"])
            .status
            .success()
    );
    let child = support::spawn(&sandbox);
    hold.wait_started();
    let call = sandbox.claude.run_calls().remove(0);
    kill_process(process::pid(child.id()), Signal::TERM).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("failed (interrupted: shutdown), requeued"),
        "{stderr}"
    );
    assert_eq!(support::text(&script), "");
    assert_eq!(
        support::count_sql(
            &sandbox,
            "SELECT count(*) FROM jobs WHERE state = 'queued' AND retry_count = 0"
        ),
        1
    );
    process::wait_until_gone(call.pid.unwrap());
    process::wait_until_gone(call.child_pid.unwrap());
}
