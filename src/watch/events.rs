mod date;
#[cfg(test)]
mod outcomes;
#[cfg(test)]
mod tests;

use crate::runner::{JobEnd, ReviewResult};
pub(crate) use date::utc;

pub(super) fn print(line: &str) {
    eprintln!(
        "{} {}",
        utc(crate::store::now_ms()),
        crate::render::sanitize(line).replace(['\n', '\r'], " ")
    );
}

pub(super) fn outcome(result: &ReviewResult) -> String {
    match result {
        ReviewResult::Succeeded {
            reused: true,
            run_id,
            ..
        } => format!("succeeded (reused run {run_id})"),
        ReviewResult::Succeeded { findings, .. } if findings.is_empty() => {
            "succeeded (no_findings)".into()
        }
        ReviewResult::Succeeded { findings, .. } => {
            format!("succeeded (findings: {})", findings.len())
        }
        ReviewResult::RunFailed {
            kind,
            message,
            job_end,
            ..
        } => failed(kind.as_str(), message, job_end),
        ReviewResult::Cancelled { job_end, .. } => failed("interrupted: shutdown", "", job_end),
        ReviewResult::JobFailed { reason } => format!("failed (cli_error): {reason}"),
    }
}

fn failed(kind: &str, message: &str, end: &JobEnd) -> String {
    match end {
        JobEnd::Failed if message.is_empty() => format!("failed ({kind})"),
        JobEnd::Failed => format!("failed ({kind}): {message}"),
        JobEnd::Requeued {
            retry: Some((retry, max)),
            next_attempt_at: Some(at),
        } => format!("failed ({kind}), retry {retry} of {max} at {}", utc(*at)),
        JobEnd::Requeued { .. } => format!("failed ({kind}), requeued"),
    }
}
