mod findings;
mod sanitize;
#[cfg(test)]
mod tests;

use crate::pr_url::PrRef;
use crate::runner::ReviewResult;
use findings::{findings, notes};
use std::io::{self, Write};

pub(crate) use sanitize::sanitize;

pub(crate) fn review(
    pr: &PrRef,
    result: &ReviewResult,
    out: &mut impl Write,
    err: &mut impl Write,
) -> io::Result<()> {
    let label = pr.label();
    match result {
        ReviewResult::Succeeded {
            head_sha,
            findings: all,
            limitation,
            future_improvements,
            warnings,
            ..
        } => {
            let head: String = head_sha.chars().take(12).collect();
            let summary = match all.len() {
                0 => "no_findings".to_string(),
                count => format!("findings: {count}"),
            };
            writeln!(out, "{label} at {head}: succeeded ({summary})")?;
            if !all.is_empty() || limitation.is_some() || !future_improvements.is_empty() {
                writeln!(out)?;
            }
            findings(out, all)?;
            notes(out, limitation.as_deref(), future_improvements)?;
            for warning in warnings {
                writeln!(err, "warning: {}", sanitize(warning).replace('\n', " "))?;
            }
            Ok(())
        }
        ReviewResult::RunFailed { kind, message, .. } => {
            let (kind, message) = (kind.as_str(), sanitize(message));
            writeln!(err, "{label}: failed ({kind}): {message}")
        }
        ReviewResult::JobFailed { reason } => {
            writeln!(err, "{label}: failed (cli_error): {}", sanitize(reason))
        }
        ReviewResult::Cancelled { run_id: Some(_) } => {
            writeln!(err, "{label}: Run cancelled; recorded as interrupted")
        }
        ReviewResult::Cancelled { run_id: None } => {
            writeln!(
                err,
                "{label}: Cancelled before the run started; the job failed"
            )
        }
    }
}
