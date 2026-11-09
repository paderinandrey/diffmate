use super::{Notice, decide, script};
use crate::agent::FailureKind;
use crate::runner::{JobEnd, ReviewResult};
use crate::store::records::JobOrigin;

fn ready(label: &str) -> Notice {
    Notice::Ready {
        label: label.to_owned(),
    }
}

fn gave_up(label: &str, reason: &str) -> Notice {
    Notice::GaveUp {
        label: label.to_owned(),
        reason: reason.to_owned(),
    }
}

fn succeeded() -> ReviewResult {
    ReviewResult::Succeeded {
        reused: false,
        run_id: 1,
        head_sha: "abc".into(),
        known_head: None,
        findings: Vec::new(),
        limitation: None,
        future_improvements: Vec::new(),
        warnings: Vec::new(),
        follow: crate::runner::Follow::Schedule,
    }
}

fn failed(message: &str, end: JobEnd) -> ReviewResult {
    ReviewResult::RunFailed {
        run_id: 1,
        kind: FailureKind::AgentError,
        message: message.to_owned(),
        job_end: end,
    }
}

fn requeued() -> JobEnd {
    JobEnd::Requeued {
        retry: None,
        next_attempt_at: None,
    }
}

#[test]
fn ready_and_give_up_titles_are_fixed() {
    assert_eq!(
        script::display(&ready("o/r#1")),
        "display notification \"o/r#1\" with title \"diffmate: draft ready\""
    );
    assert_eq!(
        script::display(&gave_up("o/r#1", "boom")),
        "display notification \"o/r#1: boom\" with title \"diffmate: review failed\""
    );
}

#[test]
fn a_quote_newline_or_backslash_cannot_break_the_script() {
    let notice = gave_up("o/r#1", "say \"x\"\r\n\\next\u{202E}\u{2028}tail");
    let script = script::display(&notice);
    assert_eq!(script.lines().count(), 1);
    assert!(!script.contains('\n'));
    assert!(!script.contains('\u{202E}'));
    assert!(script.contains("o/r#1: say \\\"x\\\" \\\\next tail"));
    assert!(script.ends_with("with title \"diffmate: review failed\""));
}

#[test]
fn only_an_automatic_terminal_result_notifies() {
    let label = "o/r#1";
    let ok = succeeded();
    assert!(matches!(
        decide(JobOrigin::Automatic, &ok, label),
        Some(Notice::Ready { .. })
    ));
    assert!(decide(JobOrigin::Manual, &ok, label).is_none());
    let terminal = failed("boom", JobEnd::Failed);
    assert!(matches!(
        decide(JobOrigin::Automatic, &terminal, label),
        Some(Notice::GaveUp { reason, .. }) if reason == "boom"
    ));
    assert!(decide(JobOrigin::Automatic, &failed("boom", requeued()), label).is_none());
    let job = ReviewResult::JobFailed {
        reason: "missing gh".into(),
    };
    assert!(matches!(
        decide(JobOrigin::Automatic, &job, label),
        Some(Notice::GaveUp { reason, .. }) if reason == "missing gh"
    ));
    assert!(decide(JobOrigin::Manual, &job, label).is_none());
    let shutdown = ReviewResult::Cancelled {
        run_id: Some(1),
        job_end: requeued(),
    };
    assert!(decide(JobOrigin::Automatic, &shutdown, label).is_none());
}
