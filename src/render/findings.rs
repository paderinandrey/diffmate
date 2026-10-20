use super::sanitize;
use crate::store::records::FindingRecord;
use std::io::{self, Write};

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

pub(super) fn findings(out: &mut impl Write, all: &[FindingRecord]) -> io::Result<()> {
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

pub(super) fn notes(
    out: &mut impl Write,
    limitation: Option<&str>,
    items: &[String],
) -> io::Result<()> {
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
