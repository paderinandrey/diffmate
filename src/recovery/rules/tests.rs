mod support;
mod table;
mod tokens;
mod transient;

use super::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn arbitrary_failure_sequences_never_exceed_the_budget(
        max in 0_u32..6, sequence in proptest::collection::vec(0_u8..3, 1..80),
    ) {
        let mut state = RetryState { retry_count: 0, max_retries: None };
        let mut retryable = 0;
        for value in sequence {
            let failure = match value { 0 => Failure::Crash, 1 => Failure::Timeout, _ => Failure::Shutdown };
            let next = after_failure(JobOrigin::Automatic, &failure, state, max, 1000);
            if value != 2 { retryable += 1; }
            match next {
                JobNext::Requeue { retry_count, max_retries, next_attempt_at } => {
                    if value == 2 {
                        prop_assert_eq!(retry_count, state.retry_count);
                        prop_assert_eq!(max_retries, state.max_retries);
                        prop_assert_eq!(next_attempt_at, None);
                    } else {
                        prop_assert_eq!(retry_count, state.retry_count + 1);
                        prop_assert_eq!(max_retries, Some(max));
                        let delay = if retry_count == 1 { 60_000 } else { 300_000 };
                        prop_assert_eq!(next_attempt_at, Some(1000 + delay));
                    }
                    prop_assert_eq!(retry_count, retryable.min(max));
                    state = RetryState { retry_count, max_retries };
                }
                JobNext::Fail { max_retries, .. } => {
                    prop_assert!(value != 2);
                    prop_assert_eq!(state.retry_count, max);
                    prop_assert_eq!(retryable, max + 1);
                    prop_assert_eq!(max_retries, Some(max));
                    break;
                }
            }
        }
    }
}
