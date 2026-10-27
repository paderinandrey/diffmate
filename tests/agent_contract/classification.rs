use super::failed_run;
use crate::support::fake_claude::{finding_json, output_json, success_envelope};
use crate::support::sandbox::Sandbox;
use crate::tracer::{count, database, review, with_pr};

#[test]
fn error_envelope_with_exit_zero_is_failed() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    let envelope = include_str!("../fixtures/claude/error-max-turns.stdout");
    sandbox.claude.set_run(envelope, 0);

    let output = review(&sandbox);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("failed (agent_error)"), "{stderr}");
    let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
    assert!(!stdout.contains("no_findings") && !stdout.contains("no findings"));
    let (kind, detail) = failed_run(&sandbox);
    assert_eq!(kind, "agent_error");
    assert!(detail.contains("error_max_turns"), "{detail}");
    assert_eq!(count(&database(&sandbox), "SELECT count(*) FROM drafts"), 0);
}

#[test]
fn status_disagreement_is_invalid_output() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    let finding = finding_json(Some("src/lib.rs"), Some(3), Some("RIGHT"), "high", "T", "B");
    let json = output_json(&[finding], None, &[]);
    let wrong = json.replace("\"status\":\"findings\"", "\"status\":\"no_findings\"");
    assert_ne!(wrong, json);
    sandbox.claude.set_run(&success_envelope(&wrong), 0);

    let output = review(&sandbox);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("failed (invalid_output)"), "{stderr}");
    assert_eq!(output.stdout, Vec::<u8>::new());
    let (kind, detail) = failed_run(&sandbox);
    assert_eq!(kind, "invalid_output");
    assert!(detail.contains("status disagrees"), "{detail}");
    assert_eq!(count(&database(&sandbox), "SELECT count(*) FROM drafts"), 0);
}
