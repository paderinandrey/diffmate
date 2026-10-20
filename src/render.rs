use crate::pr_url::PrRef;
use crate::runner::ReviewResult;
use std::io::{self, Write};

pub(crate) fn sanitize(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
        .collect()
}

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
            findings,
            limitation,
            future_improvements,
            warnings,
            ..
        } => {
            let head: String = head_sha.chars().take(12).collect();
            let summary = if findings.is_empty() {
                "no_findings".to_string()
            } else {
                format!("findings: {}", findings.len())
            };
            writeln!(out, "{label} at {head}: succeeded ({summary})\n")?;
            for finding in findings {
                let path = finding.path.as_deref().unwrap_or("(no file)");
                let place = finding
                    .line
                    .map_or_else(|| path.to_string(), |line| format!("{path}:{line}"));
                let title = sanitize(&finding.title);
                writeln!(
                    out,
                    "{} [{}] {title}",
                    sanitize(&place),
                    finding.severity.as_str()
                )?;
                writeln!(out, "{}\n", sanitize(&finding.body))?;
            }
            if let Some(limitation) = limitation {
                writeln!(out, "Limitation: {}", sanitize(limitation))?;
            }
            if !future_improvements.is_empty() {
                writeln!(out, "Future improvements:")?;
            }
            for item in future_improvements {
                writeln!(out, "- {}", sanitize(item))?;
            }
            for warning in warnings {
                writeln!(err, "warning: {}", sanitize(warning))?;
            }
            Ok(())
        }
        ReviewResult::RunFailed { kind, message, .. } => {
            writeln!(
                err,
                "{label}: failed ({}): {}",
                kind.as_str(),
                sanitize(message)
            )
        }
        ReviewResult::JobFailed { reason } => {
            writeln!(err, "{label}: failed (cli_error): {}", sanitize(reason))
        }
    }
}
