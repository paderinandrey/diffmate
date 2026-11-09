use super::super::super::{URL, debug_enqueue, with_pr};
use super::super::{config, search_hit};
use super::support::{self, Osascript};
use crate::support::fake_claude::{finding_json, output_json, success_envelope};
use crate::support::sandbox::Sandbox;
use std::fs;

const BODY: &str = "finding-body-should-stay-hidden";

fn ready_sandbox() -> (Sandbox, Osascript) {
    let sandbox = Sandbox::new();
    config(
        &sandbox,
        "poll_interval_seconds = 60\ndebounce_seconds = 1\n",
    );
    let script = support::install(&sandbox);
    with_pr(&sandbox);
    let finding = finding_json(
        Some("src/lib.rs"),
        Some(3),
        Some("RIGHT"),
        "low",
        "nit",
        BODY,
    );
    let output = output_json(&[finding], None, &[]);
    sandbox.claude.set_run(&success_envelope(&output), 0);
    sandbox.gh.route("search prs", &search_hit(false));
    (sandbox, script)
}

#[test]
fn a_ready_automatic_draft_runs_osascript() {
    let (sandbox, script) = ready_sandbox();
    let mut child = support::spawn(&sandbox);
    let stderr = support::finish(&mut child, "succeeded (findings: 1)");
    let log = support::text(&script);
    let scripts = support::scripts(&log);
    assert_eq!(scripts.len(), 1, "{log}");
    assert!(scripts[0].contains("diffmate: draft ready"), "{log}");
    assert!(scripts[0].contains("o/r#1"), "{log}");
    assert!(!scripts[0].contains(BODY), "{log}");
    assert!(log.contains("LC_ALL=C\n"), "{log}");
    assert!(log.contains("NOT_A_TOOL_VAR=\n"), "{log}");
    assert!(!log.contains("secret"), "{log}");
    assert!(log.lines().any(|line| line == "-e"), "{log}");
    assert!(!stderr.contains(BODY), "{stderr}");
    assert!(!stderr.contains("notification-failed"), "{stderr}");
    assert_eq!(
        support::count_sql(
            &sandbox,
            "SELECT count(*) FROM jobs WHERE state = 'succeeded' AND origin = 'automatic'"
        ),
        1
    );
}

#[test]
fn a_failed_osascript_still_succeeds_the_job() {
    let sandbox = Sandbox::new();
    let script = support::install(&sandbox);
    fs::write(&script.status, "1\n").unwrap();
    with_pr(&sandbox);
    assert!(
        debug_enqueue(&sandbox, URL, &["--automatic"])
            .status
            .success()
    );
    let mut child = support::spawn(&sandbox);
    let stderr = support::finish(&mut child, "succeeded");
    assert!(stderr.contains("notification-failed"), "{stderr}");
    assert_eq!(support::scripts(&support::text(&script)).len(), 1);
    assert_eq!(
        support::count_sql(
            &sandbox,
            "SELECT count(*) FROM jobs WHERE state = 'succeeded' AND retry_count = 0"
        ),
        1
    );
}

#[test]
fn a_manual_job_records_no_osascript_argv() {
    let sandbox = Sandbox::new();
    let script = support::install(&sandbox);
    with_pr(&sandbox);
    assert!(debug_enqueue(&sandbox, URL, &[]).status.success());
    let mut child = support::spawn(&sandbox);
    let stderr = support::finish(&mut child, "succeeded");
    assert!(stderr.contains("succeeded"), "{stderr}");
    assert_eq!(support::text(&script), "");
    assert_eq!(
        support::count_sql(
            &sandbox,
            "SELECT count(*) FROM jobs WHERE state = 'succeeded' AND origin = 'manual'"
        ),
        1
    );
}
