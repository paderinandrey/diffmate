use super::*;

#[test]
fn cancellation_reports_when_the_job_was_requeued() {
    let result = ReviewResult::Cancelled {
        run_id: Some(1),
        job_end: JobEnd::Requeued {
            retry: None,
            next_attempt_at: None,
        },
    };
    assert_eq!(
        render(&result),
        (
            String::new(),
            "o/r#1: Run cancelled; recorded as interrupted; the job is queued again\n".into()
        )
    );
}
