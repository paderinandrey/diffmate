use super::WATCH_IDLE_POLL;
use std::time::Duration;

pub(super) fn idle_delay(now_ms: i64, next_poll_at: i64, next_attempt_at: Option<i64>) -> Duration {
    let capped = WATCH_IDLE_POLL.min(until(now_ms, next_poll_at));
    match next_attempt_at {
        Some(at) if at > now_ms => capped.min(until(now_ms, at)),
        _ => capped,
    }
}

fn until(now_ms: i64, at: i64) -> Duration {
    match u64::try_from(at.saturating_sub(now_ms)) {
        Ok(millis) => Duration::from_millis(millis),
        Err(_) => Duration::ZERO,
    }
}
