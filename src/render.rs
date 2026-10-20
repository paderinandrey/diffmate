mod sanitize;
#[cfg(test)]
mod tests;

use crate::pr_url::PrRef;
use crate::runner::ReviewResult;
use crate::store::records::FindingRecord;
use std::io::{self, Write};

pub(crate) use sanitize::sanitize;

fn anchor(finding: &FindingRecord) -> Option<String> {
    let (path, line) = (finding.path.as_deref()?, finding.line?);
    Some(format!("{path}:{line}"))
}

fn block(out: &mut impl Write, place: &str, finding: &FindingRecord) -> io::Result<()> {
    let severity = finding.severity.as_str();
    let title = sanitize(&finding.title);
    writeln!(out, "{} [{severity}] {title}", sanitize(place))?;
    writeln!(out, "{}", sanitize(&finding.body))
}

fn findings(out: &mut impl Write, all: &[FindingRecord]) -> io::Result<()> {
    let mut unanchored = Vec::new();
    for finding in all {
        match anchor(finding) {
            Some(place) => {
                block(out, &place, finding)?;
                writeln!(out)?;
            }
            None => unanchored.push(finding),
        }
    }
    if !unanchored.is_empty() {
        writeln!(out, "Unanchored findings:")?;
    }
    for finding in unanchored {
        block(out, finding.path.as_deref().unwrap_or("(no file)"), finding)?;
    }
    Ok(())
}

fn notes(out: &mut impl Write, limitation: Option<&str>, items: &[String]) -> io::Result<()> {
    if let Some(limitation) = limitation {
        writeln!(out, "Limitation: {}", sanitize(limitation))?;
    }
    if !items.is_empty() {
        writeln!(out, "Future improvements:")?;
    }
    for item in items {
        writeln!(out, "- {}", sanitize(item))?;
    }
    Ok(())
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
    }
}
