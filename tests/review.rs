mod support;
mod tracer;

use std::fs;
use support::fake_claude::{finding_json, output_json, success_envelope};
use support::sandbox::Sandbox;
use tracer::agent_call::assert_agent_call;
use tracer::stored::assert_stored;
use tracer::{count, database, review, with_pr};

#[test]
fn tracer_review_prints_anchored_finding() {
    let sandbox = Sandbox::new();
    let (base, head) = with_pr(&sandbox);
    let body = "The bound is wrong.";
    let finding = finding_json(
        Some("src/lib.rs"),
        Some(3),
        Some("RIGHT"),
        "high",
        "Off by one",
        body,
    );
    sandbox
        .claude
        .set_run(&success_envelope(&output_json(&[finding], None, &[])), 0);

    let output = review(&sandbox);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("src/lib.rs:3 [high] Off by one") && stdout.contains(body));
    assert_stored(&sandbox, &base, &head);
    assert_agent_call(&sandbox);
}

#[test]
fn review_fails_before_run_when_gh_fails() {
    let sandbox = Sandbox::new();

    let output = review(&sandbox);

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("failed (cli_error)"));
    let db = database(&sandbox);
    let sql = "SELECT failure_reason FROM jobs WHERE state = 'failed'";
    let reason: String = db.query_row(sql, [], |r| r.get(0)).unwrap();
    assert!(reason.contains("gh"), "{reason}");
    assert_eq!(count(&db, "SELECT count(*) FROM runs"), 0);
}

#[test]
fn agent_error_envelope_stores_a_failed_run() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    let envelope = include_str!("fixtures/claude/error-max-turns.stdout");
    sandbox.claude.set_run(envelope, 1);

    let output = review(&sandbox);

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("failed (agent_error)"));
    let db = database(&sandbox);
    let sql = "SELECT failure_kind, failure_detail FROM runs WHERE state = 'failed'";
    let (kind, detail): (String, String) = db
        .query_row(sql, [], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap();
    assert_eq!(kind, "agent_error");
    assert!(
        detail.contains("error_max_turns") && detail.contains("max_turns"),
        "{detail}"
    );
    assert_eq!(count(&db, "SELECT count(*) FROM drafts"), 0);
    assert_eq!(
        fs::read_dir(sandbox.cache_dir().join("runs"))
            .unwrap()
            .count(),
        0
    );
}
