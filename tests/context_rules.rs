mod support;
#[allow(dead_code)]
mod tracer;

use std::fs;
use std::path::Path;
use support::fake_gh::pr_view_json;
use support::sandbox::Sandbox;
use tracer::{BASE_LIB, HEAD_LIB, URL, review};

fn rule(context: &Path, name: &str) -> String {
    fs::read_to_string(context.join("rules").join(name)).unwrap()
}

#[test]
fn rules_are_read_at_merge_base() {
    let sandbox = Sandbox::new();
    let upstream = sandbox.upstream("o", "r");
    let base_files = [
        ("src/lib.rs", BASE_LIB),
        ("AGENTS.md", "base rules\n"),
        ("CONTRIBUTING.md", "contributing text\n"),
    ];
    upstream.commit(&base_files, "base");
    upstream.commit_symlink("CLAUDE.md", "AGENTS.md", "link");
    upstream.push_branch("main");
    let head_files = [
        ("src/lib.rs", HEAD_LIB),
        ("AGENTS.md", "pr rules\n"),
        ("REVIEW.md", "pr review rules\n"),
    ];
    let head = upstream.pr_head(1, "main", &head_files);
    let json = pr_view_json("o", "r", 1, &head, "main", false);
    sandbox.gh.route(&format!("pr view {URL}"), &json);

    let output = review(&sandbox);

    assert!(output.status.success(), "{output:?}");
    let calls = sandbox.claude.run_calls();
    let context = calls[0].context.as_ref().unwrap();
    assert_eq!(rule(context, "AGENTS.md"), "base rules\n");
    let linked = rule(context, "CLAUDE.md");
    let note = "CLAUDE.md is a symlink to AGENTS.md at the merge-base; the content of AGENTS.md follows.\n";
    assert_eq!(linked, format!("{note}base rules\n"));
    assert_eq!(rule(context, "CONTRIBUTING.md"), "contributing text\n");
    let absent = rule(context, "REVIEW.md");
    assert_eq!(absent, "REVIEW.md is not present at the merge-base.\n");
    for name in ["AGENTS.md", "CLAUDE.md", "CONTRIBUTING.md", "REVIEW.md"] {
        assert!(!rule(context, name).contains("pr rules"), "{name}");
    }
}

#[test]
fn unsafe_missing_and_oversized_rules_are_marked_not_followed() {
    let sandbox = Sandbox::new();
    let upstream = sandbox.upstream("o", "r");
    let big = "x\n".repeat(102_400);
    let files = [
        ("src/lib.rs", BASE_LIB),
        ("CONTRIBUTING.md", big.as_str()),
        ("REVIEW.md/inner", "inside a directory\n"),
    ];
    upstream.commit(&files, "base");
    upstream.commit_symlink("CLAUDE.md", "../outside", "outside link");
    upstream.commit_symlink("AGENTS.md", "CLAUDE.md", "second hop");
    upstream.push_branch("main");
    let head = upstream.pr_head(1, "main", &[("src/lib.rs", HEAD_LIB)]);
    let json = pr_view_json("o", "r", 1, &head, "main", false);
    sandbox.gh.route(&format!("pr view {URL}"), &json);

    let output = review(&sandbox);

    assert!(output.status.success(), "{output:?}");
    let calls = sandbox.claude.run_calls();
    let context = calls[0].context.as_ref().unwrap();
    let outside = "CLAUDE.md is a symlink to ../outside, which was not followed.\n";
    assert_eq!(rule(context, "CLAUDE.md"), outside);
    let second = "AGENTS.md is a symlink to CLAUDE.md, which was not followed.\n";
    assert_eq!(rule(context, "AGENTS.md"), second);
    let directory = "REVIEW.md is not a regular file at the merge-base.\n";
    assert_eq!(rule(context, "REVIEW.md"), directory);
    let marker = "TRUNCATED: rules/CONTRIBUTING.md kept 131072 of 204800 bytes.\n";
    let capped = rule(context, "CONTRIBUTING.md");
    assert_eq!(capped.len(), marker.len() + 131_072);
    assert!(capped.starts_with(marker));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("warning: rules/CONTRIBUTING.md exceeded its size cap"));
}
