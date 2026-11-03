use crate::support::sandbox::Sandbox;
use crate::tracer::{URL, count, database, review, with_pr};
use crate::waiting::succeeded;
use std::process::{Child, Stdio};

fn spawn(sandbox: &Sandbox) -> Child {
    sandbox
        .diffmate()
        .args(["review", URL])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

#[test]
fn base_move_during_run_is_not_stale() {
    let sandbox = Sandbox::new();
    let (_, head) = with_pr(&sandbox);
    let hold = sandbox.claude.hold_run(1);
    let child = spawn(&sandbox);
    hold.wait_started();
    let upstream = sandbox.upstream("o", "r");
    upstream.commit(&[("base-only.txt", "base change")], "advance base");
    upstream.push_branch("main");
    hold.release();
    let first = child.wait_with_output().unwrap();
    succeeded(&first);
    let second = review(&sandbox);
    succeeded(&second);
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(
        String::from_utf8(first.stdout).unwrap(),
        format!("o/r#1 at {}: succeeded (no_findings)\n", &head[..12])
    );
    assert!(String::from_utf8_lossy(&second.stderr).contains("reused run 1"));
    assert_eq!(count(&database(&sandbox), "SELECT count(*) FROM runs"), 1);
    assert_eq!(sandbox.claude.run_calls().len(), 1);
}

#[test]
fn failed_recheck_keeps_the_run() {
    let sandbox = Sandbox::new();
    let (_, head) = with_pr(&sandbox);
    let hold = sandbox.claude.hold_run(1);
    let child = spawn(&sandbox);
    hold.wait_started();
    sandbox
        .gh
        .replace_route_fail(&format!("pr view {URL}"), "offline", 1);
    hold.release();
    let output = child.wait_with_output().unwrap();
    succeeded(&output);
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("o/r#1 at {}: succeeded (no_findings)\n", &head[..12])
    );
    let warning = "could not re-check the PR head: ";
    assert!(String::from_utf8_lossy(&output.stderr).contains(&format!("warning: {warning}")));
    let (known, state, warnings): (String, String, String) = database(&sandbox)
        .query_row(
            "SELECT t.head_sha, r.state, r.warnings
             FROM targets t JOIN runs r ON r.target_id = t.id",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(known, head);
    assert_eq!(state, "succeeded");
    let warnings: Vec<String> = serde_json::from_str(&warnings).unwrap();
    assert!(warnings.iter().any(|value| value.starts_with(warning)));
}
