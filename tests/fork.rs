mod support;
#[allow(dead_code)]
mod tracer;

use support::fake_claude::{finding_json, output_json, success_envelope};
use support::fake_gh::pr_view_json;
use support::git_in;
use support::sandbox::Sandbox;
use tracer::{BASE_LIB, HEAD_LIB, count, database};

const URL: &str = "https://github.com/o/r/pull/7";

fn repo(sandbox: &Sandbox) -> std::path::PathBuf {
    sandbox.state_dir().join("repos/o/r.git")
}

#[test]
fn fork_pr_head_only_under_pull_ref_is_reviewed() {
    let sandbox = Sandbox::new();
    let upstream = sandbox.upstream("o", "r");
    upstream.commit(&[("src/lib.rs", BASE_LIB)], "base");
    upstream.push_branch("main");
    let head = upstream.pr_head(7, "main", &[("src/lib.rs", HEAD_LIB)]);
    let json = pr_view_json("o", "r", 7, &head, "main", true);
    let json = json.replace(
        "\"headRefName\":\"feature\"",
        "\"headRefName\":\"fork-branch\"",
    );
    sandbox.gh.route(&format!("pr view {URL}"), &json);
    let finding = finding_json(
        Some("src/lib.rs"),
        Some(3),
        Some("RIGHT"),
        "high",
        "Off by one",
        "Bound.",
    );
    sandbox
        .claude
        .set_run(&success_envelope(&output_json(&[finding], None, &[])), 0);
    let reachable = git_in(
        &upstream.bare,
        &sandbox.home,
        &["branch", "--contains", &head],
    );
    assert_eq!(reachable, "");

    let output = sandbox.run(&["review", URL]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert!(String::from_utf8_lossy(&output.stdout).contains("src/lib.rs:3 [high] Off by one"));
    let sql = "SELECT head_sha FROM runs WHERE state = 'succeeded'";
    let stored: String = database(&sandbox).query_row(sql, [], |r| r.get(0)).unwrap();
    assert_eq!(stored, head);
    let fetched = ["rev-parse", "refs/diffmate/pull/7/head"];
    assert_eq!(git_in(&repo(&sandbox), &sandbox.home, &fetched), head);
}

#[test]
fn unsafe_base_branch_fails_before_fetch() {
    let sandbox = Sandbox::new();
    let json = pr_view_json(
        "o",
        "r",
        7,
        "0123456789012345678901234567890123456789",
        "main:refs/heads/evil",
        false,
    );
    sandbox.gh.route(&format!("pr view {URL}"), &json);

    let output = sandbox.run(&["review", URL]);

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("failed (cli_error)") && stderr.contains("main:refs/heads/evil"),
        "{stderr}"
    );
    let db = database(&sandbox);
    assert_eq!(
        count(&db, "SELECT count(*) FROM jobs WHERE state = 'failed'"),
        1
    );
    assert_eq!(count(&db, "SELECT count(*) FROM runs"), 0);
    assert_eq!(
        git_in(&repo(&sandbox), &sandbox.home, &["for-each-ref"]),
        ""
    );
}
