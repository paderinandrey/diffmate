#[path = "review_output/helpers.rs"]
mod helpers;
#[path = "review_output/render.rs"]
mod render;
mod support;
#[allow(dead_code)]
mod tracer;

use helpers::{MOVED, anchored, finding_rows, review_one, row, run_warnings, texts};
use support::sandbox::Sandbox;
use tracer::{with_pr, with_pr_head};

#[test]
fn out_of_hunk_finding_moves_to_review_body() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);

    let output = review_one(&sandbox, anchored(50, "RIGHT", "high", "Bad bound"));

    let (stdout, stderr) = texts(&output);
    assert!(output.status.success(), "{stderr}");
    let listed = "Unanchored findings:\nsrc/lib.rs [high] Bad bound\nBody.\n";
    assert!(
        stdout.contains(listed) && !stdout.contains("src/lib.rs:50"),
        "{stdout}"
    );
    assert!(stderr.contains(&format!("warning: {MOVED}")), "{stderr}");
    assert_eq!(finding_rows(&sandbox), [row(None, None)]);
    assert_eq!(run_warnings(&sandbox), [MOVED]);
}

#[test]
fn left_anchor_on_removed_line_is_kept() {
    let sandbox = Sandbox::new();
    with_pr_head(&sandbox, "fn a() {}\nfn c() {}\n");

    let output = review_one(&sandbox, anchored(2, "LEFT", "low", "Dropped fn"));

    let (stdout, stderr) = texts(&output);
    assert!(output.status.success(), "{stderr}");
    assert!(stdout.contains("src/lib.rs:2 [low] Dropped fn"), "{stdout}");
    assert_eq!(finding_rows(&sandbox), [row(Some(2), Some("LEFT"))]);
    assert_eq!(run_warnings(&sandbox), Vec::<String>::new());
}

#[test]
fn unknown_previous_finding_id_is_nulled() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    let finding = anchored(3, "RIGHT", "low", "Again");
    let null = "\"previous_finding_id\":null";
    let linked = finding.replace(null, "\"previous_finding_id\":\"9.1\"");
    assert_ne!(linked, finding);

    let output = review_one(&sandbox, linked);

    let (_, stderr) = texts(&output);
    let unknown = "finding 1: previous_finding_id 9.1 does not refer to a finding in previous.json; treated as new";
    assert!(output.status.success(), "{stderr}");
    assert!(stderr.contains(&format!("warning: {unknown}")), "{stderr}");
    assert_eq!(finding_rows(&sandbox), [row(Some(3), Some("RIGHT"))]);
    assert_eq!(run_warnings(&sandbox), [unknown]);
}
