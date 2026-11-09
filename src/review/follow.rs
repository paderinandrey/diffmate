use crate::runner::{JobEnd, ReviewResult};

pub(super) fn again(result: &ReviewResult) -> bool {
    matches!(
        result,
        ReviewResult::RunFailed {
            job_end: JobEnd::Requeued { .. },
            ..
        }
    )
}

#[cfg(test)]
mod tests {
    use super::again;
    use crate::agent::FailureKind;
    use crate::runner::{JobEnd, ReviewResult};

    fn requeued() -> JobEnd {
        JobEnd::Requeued {
            retry: Some((1, 2)),
            next_attempt_at: Some(1),
        }
    }

    #[test]
    fn a_requeued_timeout_is_served_again() {
        let result = ReviewResult::RunFailed {
            run_id: 1,
            kind: FailureKind::Timeout,
            message: "stopped by the run timeout".into(),
            job_end: requeued(),
        };
        assert!(again(&result));
    }

    #[test]
    fn a_terminal_failure_ends_the_review() {
        let result = ReviewResult::RunFailed {
            run_id: 1,
            kind: FailureKind::Timeout,
            message: "stopped by the run timeout".into(),
            job_end: JobEnd::Failed,
        };
        assert!(!again(&result));
    }

    #[test]
    fn a_cancelled_requeue_ends_the_review() {
        let result = ReviewResult::Cancelled {
            run_id: Some(1),
            job_end: requeued(),
        };
        assert!(!again(&result));
    }
}
