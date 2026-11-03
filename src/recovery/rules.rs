mod failure;
#[cfg(test)]
mod tests;
use crate::store::records::JobOrigin;
pub(crate) use failure::Failure;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RetryState {
    pub retry_count: u32,
    pub max_retries: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum JobNext {
    Requeue {
        retry_count: u32,
        max_retries: Option<u32>,
        next_attempt_at: Option<i64>,
    },
    Fail {
        reason: String,
        max_retries: Option<u32>,
    },
}

pub(crate) fn retry_delay(retry_number: u32) -> Duration {
    Duration::from_secs(if retry_number == 1 { 60 } else { 300 })
}

pub(crate) fn after_failure(
    origin: JobOrigin,
    failure: &Failure,
    state: RetryState,
    config_max: u32,
    now_ms: i64,
) -> JobNext {
    let RetryState {
        retry_count,
        max_retries,
    } = state;
    let mut reason = failure.text().to_string();
    let mut budget = max_retries;
    if origin == JobOrigin::Automatic {
        if matches!(failure, Failure::Shutdown) {
            return JobNext::Requeue {
                retry_count,
                max_retries,
                next_attempt_at: None,
            };
        }
        if matches!(failure, Failure::Crash | Failure::Timeout) {
            let max = max_retries.unwrap_or(config_max);
            budget = Some(max);
            if retry_count < max {
                let retry_count = retry_count + 1;
                let delay = i64::try_from(retry_delay(retry_count).as_millis()).unwrap_or(i64::MAX);
                return JobNext::Requeue {
                    retry_count,
                    max_retries: budget,
                    next_attempt_at: Some(now_ms.saturating_add(delay)),
                };
            }
            reason = format!("{reason}; no retries left ({max} of {max} used)");
        }
    }
    JobNext::Fail {
        reason,
        max_retries: budget,
    }
}
