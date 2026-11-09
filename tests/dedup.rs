mod support;
#[allow(dead_code)]
mod tracer;

use support::fake_claude::{finding_json, output_json, success_envelope};
use support::sandbox::Sandbox;
use tracer::{URL, count, database, review, with_pr};

#[test]
fn same_head_reuses_the_run() {
    let sandbox = Sandbox::new();
    let (_, head) = with_pr(&sandbox);
    let finding = finding_json(
        Some("src/lib.rs"),
        Some(3),
        Some("RIGHT"),
        "high",
        "First",
        "Body.",
    );
    sandbox
        .claude
        .set_run(&success_envelope(&output_json(&[finding], None, &[])), 0);
    let first = review(&sandbox);
    assert!(first.status.success(), "{first:?}");
    let second = review(&sandbox);
    assert!(second.status.success(), "{second:?}");
    assert_eq!(first.stdout, second.stdout);
    let db = database(&sandbox);
    assert_eq!(count(&db, "SELECT count(*) FROM runs"), 1);
    assert_eq!(count(&db, "SELECT count(*) FROM jobs"), 2);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE state = 'succeeded' AND result_run_id = 1"
        ),
        2
    );
    assert_eq!(sandbox.claude.run_calls().len(), 1);
    let status = sandbox.run(&["status"]);
    assert!(status.status.success());
    assert!(String::from_utf8_lossy(&status.stdout).contains("succeeded (findings: 1)"));
    assert!(String::from_utf8_lossy(&second.stderr).contains(&format!(
        "reused run 1 for head {}; --force starts a new run",
        &head[..12]
    )));
}

#[test]
fn force_makes_a_new_run() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    let finding = finding_json(
        Some("src/lib.rs"),
        Some(3),
        Some("RIGHT"),
        "high",
        "First",
        "Body.",
    );
    sandbox
        .claude
        .set_run(&success_envelope(&output_json(&[finding], None, &[])), 0);
    assert!(review(&sandbox).status.success());
    assert!(review(&sandbox).status.success());
    sandbox
        .claude
        .set_run(&success_envelope(&output_json(&[], None, &[])), 0);
    let forced = sandbox.run(&["review", URL, "--force"]);
    assert!(forced.status.success(), "{forced:?}");
    let db = database(&sandbox);
    assert_eq!(count(&db, "SELECT count(*) FROM runs"), 2);
    assert_eq!(count(&db, "SELECT count(*) FROM drafts"), 2);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM findings WHERE run_id = 1 AND title = 'First'"
        ),
        1
    );
    assert_eq!(
        count(&db, "SELECT count(*) FROM findings WHERE run_id = 2"),
        0
    );
    assert_eq!(sandbox.claude.run_calls().len(), 2);
}
