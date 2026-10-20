#[path = "review_output/helpers.rs"]
mod helpers;
mod support;
#[allow(dead_code)]
mod tracer;

use helpers::{MOVED, finding_rows, review_with, run_warnings, texts};
use support::fake_claude::{finding_json, output_json};
use support::sandbox::Sandbox;
use tracer::{with_pr, with_pr_head};

#[test]
fn out_of_hunk_finding_moves_to_review_body() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    let finding = finding_json(
        Some("src/lib.rs"),
        Some(50),
        Some("RIGHT"),
        "high",
        "Bad bound",
        "Body.",
    );

    let output = review_with(&sandbox, &output_json(&[finding], None, &[]));

    let (stdout, stderr) = texts(&output);
    assert!(output.status.success(), "{stderr}");
    assert!(stdout.contains("src/lib.rs [high] Bad bound") && !stdout.contains("src/lib.rs:50"));
    assert!(stderr.contains(&format!("warning: {MOVED}")), "{stderr}");
    assert_eq!(
        finding_rows(&sandbox),
        [(Some("src/lib.rs".into()), None, None, None)]
    );
    assert_eq!(run_warnings(&sandbox), [MOVED]);
}

#[test]
fn left_anchor_on_removed_line_is_kept() {
    let sandbox = Sandbox::new();
    with_pr_head(&sandbox, "fn a() {}\nfn c() {}\n");
    let finding = finding_json(
        Some("src/lib.rs"),
        Some(2),
        Some("LEFT"),
        "low",
        "Dropped fn",
        "Body.",
    );

    let output = review_with(&sandbox, &output_json(&[finding], None, &[]));

    let (stdout, stderr) = texts(&output);
    assert!(output.status.success(), "{stderr}");
    assert!(stdout.contains("src/lib.rs:2 [low] Dropped fn"), "{stdout}");
    let kept = (
        Some("src/lib.rs".into()),
        Some(2),
        Some("LEFT".into()),
        None,
    );
    assert_eq!(finding_rows(&sandbox), [kept]);
    assert_eq!(run_warnings(&sandbox), Vec::<String>::new());
}

#[test]
fn unknown_previous_finding_id_is_nulled() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    let finding = finding_json(
        Some("src/lib.rs"),
        Some(3),
        Some("RIGHT"),
        "low",
        "Again",
        "Body.",
    );
    let linked = finding.replace(
        "\"previous_finding_id\":null",
        "\"previous_finding_id\":\"9.1\"",
    );
    assert_ne!(linked, finding);

    let output = review_with(&sandbox, &output_json(&[linked], None, &[]));

    let (_, stderr) = texts(&output);
    let unknown = "finding 1: previous_finding_id 9.1 does not refer to a finding in previous.json; treated as new";
    assert!(
        output.status.success() && stderr.contains(&format!("warning: {unknown}")),
        "{stderr}"
    );
    let kept = (
        Some("src/lib.rs".into()),
        Some(3),
        Some("RIGHT".into()),
        None,
    );
    assert_eq!(finding_rows(&sandbox), [kept]);
    assert_eq!(run_warnings(&sandbox), [unknown]);
}
