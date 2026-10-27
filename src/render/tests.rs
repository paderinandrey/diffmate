mod fixture;
mod sanitize;

use crate::agent::{FailureKind, Severity};
use crate::runner::ReviewResult;
use fixture::{finding, render, succeeded};

#[test]
fn findings_print_anchored_first_then_unanchored_then_notes() {
    let findings = vec![
        finding(Some("src/a.rs"), Some(3), Severity::High, "First"),
        finding(Some("src/x.rs"), None, Severity::Medium, "Third"),
        finding(Some("src/b.rs"), Some(9), Severity::Low, "Second"),
        finding(None, None, Severity::Low, "Fourth"),
    ];

    let (out, err) = render(&succeeded(
        findings,
        Some("CI data missing"),
        &["Add tests", "Add docs"],
    ));

    let expected = "o/r#1 at 0123456789ab: succeeded (findings: 4)\n\nsrc/a.rs:3 [high] First\nBody of First\n\nsrc/b.rs:9 [low] Second\nBody of Second\n\nUnanchored findings:\nsrc/x.rs [medium] Third\nBody of Third\n(no file) [low] Fourth\nBody of Fourth\nLimitation: CI data missing\nFuture improvements:\n- Add tests\n- Add docs\n";
    assert_eq!((out.as_str(), err.as_str()), (expected, ""));
}

#[test]
fn no_findings_prints_status_and_only_the_notes_that_exist() {
    let (bare, _) = render(&succeeded(Vec::new(), None, &[]));
    assert_eq!(bare, "o/r#1 at 0123456789ab: succeeded (no_findings)\n");

    let (noted, _) = render(&succeeded(Vec::new(), Some("CI data missing"), &["Add CI"]));

    let expected = "o/r#1 at 0123456789ab: succeeded (no_findings)\n\nLimitation: CI data missing\nFuture improvements:\n- Add CI\n";
    assert_eq!(noted, expected);
}

#[test]
fn warnings_go_to_the_error_writer_one_line_each() {
    let mut result = succeeded(Vec::new(), None, &[]);
    if let ReviewResult::Succeeded { warnings, .. } = &mut result {
        warnings.extend(["first".to_string(), "se\u{1b}cond\nline".to_string()]);
    }

    let (_, err) = render(&result);

    assert_eq!(err, "warning: first\nwarning: second line\n");
}

#[test]
fn failures_never_claim_there_were_no_findings() {
    let run = ReviewResult::RunFailed {
        run_id: 1,
        kind: FailureKind::AgentError,
        message: "boom".into(),
    };
    let job = ReviewResult::JobFailed {
        reason: "gh failed".into(),
    };

    for (result, text) in [
        (run, "failed (agent_error): boom"),
        (job, "failed (cli_error): gh failed"),
        (
            ReviewResult::Cancelled { run_id: Some(1) },
            "Run cancelled; recorded as interrupted",
        ),
        (
            ReviewResult::Cancelled { run_id: None },
            "Cancelled before the run started; the job failed",
        ),
    ] {
        let (out, err) = render(&result);
        assert_eq!(out, "");
        assert_eq!(err, format!("o/r#1: {text}\n"));
    }
}
