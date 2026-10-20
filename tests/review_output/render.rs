use crate::helpers::{finding_rows, review_with, texts};
use crate::support::fake_claude::{finding_json, output_json};
use crate::support::sandbox::Sandbox;
use crate::tracer::with_pr;

#[test]
fn no_findings_prints_status_and_notes() {
    let sandbox = Sandbox::new();
    let (_, head) = with_pr(&sandbox);
    let json = output_json(&[], Some("CI data missing"), &["Add CI data"]);

    let output = review_with(&sandbox, &json);

    let (stdout, stderr) = texts(&output);
    assert!(output.status.success(), "{stderr}");
    let header = format!("o/r#1 at {}: succeeded (no_findings)", &head[..12]);
    let notes = "Limitation: CI data missing\nFuture improvements:\n- Add CI data";
    assert_eq!(stdout, format!("{header}\n\n{notes}\n"));
    assert_eq!(finding_rows(&sandbox), []);
}

#[test]
fn terminal_control_characters_are_stripped() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    let title = finding_json(
        Some("src/lib.rs"),
        Some(3),
        Some("RIGHT"),
        "low",
        "ESCMARK red",
        "Body\u{202e}gnp.",
    );
    let raw = title.replace("ESCMARK", "\\u001b[31m");
    assert_ne!(raw, title);

    let output = review_with(&sandbox, &output_json(&[raw], None, &[]));

    let (stdout, stderr) = texts(&output);
    assert!(output.status.success(), "{stderr}");
    assert!(
        stdout.contains("[31m red") && stdout.contains("Bodygnp."),
        "{stdout}"
    );
    assert!(!stdout.contains('\u{1b}') && !stdout.contains('\u{202e}'));
    let db = crate::tracer::database(&sandbox);
    let sql = "SELECT title, body FROM findings";
    let (stored_title, stored_body): (String, String) = db
        .query_row(sql, [], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap();
    assert!(stored_title.contains('\u{1b}') && stored_body.contains('\u{202e}'));
}
