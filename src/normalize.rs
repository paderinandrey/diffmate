#[cfg(test)]
mod tests;

use crate::agent::ReviewOutput;
use crate::diff::DiffIndex;
use crate::store::records::FindingRecord;
use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct Normalized {
    pub findings: Vec<FindingRecord>,
    pub warnings: Vec<String>,
}

pub(crate) fn normalize(
    output: &ReviewOutput,
    index: &DiffIndex,
    previous_ids: &HashSet<String>,
) -> Normalized {
    let mut normalized = Normalized {
        findings: Vec::new(),
        warnings: Vec::new(),
    };
    for (position, finding) in output.findings.iter().enumerate() {
        let number = position + 1;
        let mut record = FindingRecord {
            severity: finding.severity,
            path: finding.path.clone(),
            line: finding.line,
            side: finding.side,
            title: finding.title.clone(),
            body: finding.body.clone(),
            previous_finding_id: finding.previous_finding_id.clone(),
        };
        if let Some(line) = finding.line {
            let anchored = match (finding.path.as_deref(), finding.side) {
                (Some(path), Some(side)) => index.contains(path, side, line),
                _ => false,
            };
            if !anchored {
                let path = finding.path.as_deref().unwrap_or("(no file)");
                let side = finding.side.map_or("no side", |side| side.as_str());
                normalized.warnings.push(format!(
                    "finding {number}: {path}:{line} ({side}) is not inside a hunk of diff.patch; moved to the review body"
                ));
                record.line = None;
            }
        }
        if record.line.is_none() {
            record.side = None;
        }
        if let Some(id) = &finding.previous_finding_id
            && !previous_ids.contains(id)
        {
            normalized.warnings.push(format!(
                "finding {number}: previous_finding_id {id} does not refer to a finding in previous.json; treated as new"
            ));
            record.previous_finding_id = None;
        }
        normalized.findings.push(record);
    }
    normalized
}
