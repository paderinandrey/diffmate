use super::when;
use crate::store::status::RetryView;

pub(super) fn describe(retry: &RetryView, now_ms: i64) -> Option<String> {
    match retry.job_state.as_deref() {
        Some("queued") if retry.has_failed_run => match retry.next_attempt_at {
            Some(at) => {
                let maximum = retry.max_retries?;
                let timing = if at <= now_ms {
                    "due".to_string()
                } else {
                    when::until(at.saturating_sub(now_ms))
                };
                Some(format!("retry {} of {maximum} {timing}", retry.retry_count))
            }
            None => Some("requeued".to_string()),
        },
        Some("failed") if retry.origin.as_deref() == Some("automatic") => {
            let maximum = retry.max_retries?;
            (retry.retry_count >= maximum)
                .then(|| format!("no retries left ({} of {maximum} used)", retry.retry_count))
        }
        _ => None,
    }
}
