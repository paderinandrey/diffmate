mod settle;

use std::borrow::Cow;
use std::time::Duration;

use super::checks::status_token;
use crate::error::Error;
use crate::gh::{Gh, ReviewRequest};
use crate::pr_url::PrRef;
use crate::store::Store;

const LIMIT: Duration = Duration::from_secs(30);

enum Outcome {
    Submitted(String),
    Rejected(String),
    Unknown(Cow<'static, str>),
}

pub(super) async fn finish(
    store: &Store,
    gh: &impl Gh,
    pr: &PrRef,
    request: &ReviewRequest,
    job_id: i64,
    publication_id: &str,
) -> Result<(), Error> {
    let posted = tokio::time::timeout(LIMIT, gh.create_review(pr, request)).await;
    let outcome = match posted {
        Ok(Ok(created)) => Outcome::Submitted(created.id),
        Ok(Err(error)) => failed(&error),
        Err(_) => Outcome::Unknown(format!("timed out after {}s", LIMIT.as_secs()).into()),
    };
    settle::settle(store, job_id, publication_id, outcome).await
}

fn failed(error: &Error) -> Outcome {
    let reason = match error {
        Error::Command { detail, .. } => {
            if let Some(code) = status_token(detail) {
                let reason = format!("HTTP {code}: {}", http_reason(code));
                return if (400..500).contains(&code) && !matches!(code, 408 | 429) {
                    Outcome::Rejected(reason)
                } else {
                    Outcome::Unknown(reason.into())
                };
            }
            if [
                "timeout",
                "timed out",
                "deadline exceeded",
                "Timeout exceeded",
            ]
            .iter()
            .any(|word| detail.contains(word))
            {
                "request timed out"
            } else {
                "no confirmed response"
            }
        }
        Error::Io { source, .. } if source.kind() == std::io::ErrorKind::TimedOut => {
            "request timed out"
        }
        Error::Json(_) => "unreadable review response",
        _ => "no confirmed response",
    };
    Outcome::Unknown(reason.into())
}

fn http_reason(code: u16) -> &'static str {
    match code {
        400 => "bad request",
        401 => "unauthorized",
        403 => "forbidden",
        404 => "not found",
        408 => "request timeout",
        409 => "conflict",
        422 => "validation failed",
        429 => "too many requests",
        500 => "internal server error",
        502 => "bad gateway",
        503 => "service unavailable",
        504 => "gateway timeout",
        _ if (400..500).contains(&code) => "client error",
        _ if (500..600).contains(&code) => "server error",
        _ => "unexpected response",
    }
}
