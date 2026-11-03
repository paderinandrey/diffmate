use super::*;
use crate::agent::{FailureKind, Severity};
use crate::store::records::FindingRecord;

#[test]
fn failure_events_show_terminal_requeue_and_retry_outcomes() {
    let result = ReviewResult::RunFailed {
        run_id: 1,
        kind: FailureKind::Timeout,
        message: "deadline".into(),
        job_end: JobEnd::Requeued {
            retry: Some((1, 2)),
            next_attempt_at: Some(0),
        },
    };
    assert_eq!(
        outcome(&result),
        "failed (timeout), retry 1 of 2 at 1970-01-01T00:00:00Z"
    );
    assert_eq!(
        failed("cli_error", "setup", &JobEnd::Failed),
        "failed (cli_error): setup"
    );
    let result = ReviewResult::JobFailed {
        reason: "missing gh".into(),
    };
    assert_eq!(outcome(&result), "failed (cli_error): missing gh");
    let result = ReviewResult::Cancelled {
        run_id: Some(1),
        job_end: JobEnd::Failed,
    };
    assert_eq!(outcome(&result), "failed (interrupted: shutdown)");
    assert_eq!(
        failed(
            "interrupted: shutdown",
            "",
            &JobEnd::Requeued {
                retry: None,
                next_attempt_at: None
            }
        ),
        "failed (interrupted: shutdown), requeued"
    );
}

#[test]
fn successful_events_count_findings_without_printing_them_and_identify_reuse() {
    let findings = vec![FindingRecord {
        severity: Severity::High,
        path: None,
        line: None,
        side: None,
        title: "private finding text".into(),
        body: "private body".into(),
        previous_finding_id: None,
    }];
    let mut result = ReviewResult::Succeeded {
        reused: false,
        run_id: 7,
        head_sha: "head".into(),
        known_head: None,
        findings,
        limitation: None,
        future_improvements: vec![],
        warnings: vec![],
    };
    assert_eq!(outcome(&result), "succeeded (findings: 1)");
    if let ReviewResult::Succeeded { reused, .. } = &mut result {
        *reused = true;
    }
    assert_eq!(outcome(&result), "succeeded (reused run 7)");
}
