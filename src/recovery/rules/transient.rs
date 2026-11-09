use crate::agent::FailureDetail;

const TOKENS: &[&str] = &[
    "overload",
    "rate limit",
    "rate_limit",
    "too many requests",
    "network",
    "connection reset",
    "connection refused",
    "server error",
    "timed out",
    "timeout",
    "econnreset",
    "econnrefused",
];

pub(crate) fn is_transient(detail: &FailureDetail) -> bool {
    match detail
        .api_error_status
        .as_ref()
        .filter(|status| !status.is_null())
    {
        None => mentions_token(detail),
        Some(status) => status.as_u64().is_some_and(retryable_status),
    }
}

const fn retryable_status(status: u64) -> bool {
    matches!(status, 408 | 429 | 500..=599)
}

fn has_token(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    TOKENS.iter().any(|token| lower.contains(token))
}

fn mentions_token(detail: &FailureDetail) -> bool {
    has_token(&detail.message)
        || detail
            .errors
            .as_ref()
            .is_some_and(|errors| has_token(&errors.to_string()))
        || detail.subtype.as_deref().is_some_and(has_token)
        || detail
            .terminal_reason
            .as_ref()
            .is_some_and(|reason| has_token(&reason.to_string()))
}
