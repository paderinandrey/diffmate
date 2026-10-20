#[path = "context/helpers.rs"]
mod helpers;
mod support;
#[allow(dead_code)]
mod tracer;

use helpers::{
    CHECK_RUNS, add_github_routes, assert_read_only, captured, finding, read, set_output,
};
use serde_json::Value;
use support::sandbox::Sandbox;
use tracer::{count, database, review, with_pr};

#[test]
fn context_files_hold_pr_data_discussion_and_ci() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    add_github_routes(&sandbox);

    let output = review(&sandbox);

    assert!(output.status.success(), "{output:?}");
    let context = captured(&sandbox, 0);
    assert!(read(&context, "pr.md").contains("Title: Fixture PR"));
    let discussion = read(&context, "discussion.md");
    assert!(discussion.contains("alice") && discussion.contains("Please check the bound."));
    assert!(
        discussion.contains("### src/lib.rs:3 [resolved]"),
        "{discussion}"
    );
    let ci = read(&context, "ci.md");
    assert!(
        ci.contains("- build: completed, failure\n- lint: completed, success"),
        "{ci}"
    );
    assert!(
        ci.contains("src/lib.rs:3 [failure] mismatched types"),
        "{ci}"
    );
    let previous: Value = serde_json::from_str(&read(&context, "previous.json")).unwrap();
    assert_eq!(previous["available"], false);
}

#[test]
fn ci_unavailable_is_written_as_unknown() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    sandbox.gh.route_fail(CHECK_RUNS, "HTTP 502", 1);

    let output = review(&sandbox);

    assert!(output.status.success(), "{output:?}");
    let ci = read(&captured(&sandbox, 0), "ci.md");
    assert!(ci.starts_with("CI status unknown: "), "{ci}");
    assert!(!ci.to_lowercase().contains("passing"));
}

#[test]
fn previous_json_lists_earlier_findings() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    set_output(&sandbox, finding("\"previous_finding_id\":null"));
    assert!(review(&sandbox).status.success());
    set_output(&sandbox, finding("\"previous_finding_id\":\"1.1\""));

    let second = review(&sandbox);

    let stderr = String::from_utf8_lossy(&second.stderr);
    assert!(
        second.status.success() && !stderr.contains("warning:"),
        "{stderr}"
    );
    let previous: Value =
        serde_json::from_str(&read(&captured(&sandbox, 1), "previous.json")).unwrap();
    assert_eq!(previous["available"], true);
    assert_eq!(previous["findings"][0]["id"], "1.1");
    assert_eq!(previous["findings"][0]["decision"], "undecided");
    assert_eq!(previous["findings"][0]["thread_status"], "unknown");
    let db = database(&sandbox);
    let sql = "SELECT previous_finding_id FROM findings WHERE id = '2.1'";
    let linked: String = db.query_row(sql, [], |row| row.get(0)).unwrap();
    assert_eq!(linked, "1.1");
    assert_eq!(count(&db, "SELECT count(*) FROM findings"), 2);
}

#[test]
fn gh_calls_are_read_only() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    add_github_routes(&sandbox);

    let output = review(&sandbox);

    assert!(output.status.success(), "{output:?}");
    assert_read_only(&sandbox.gh.calls());
}
