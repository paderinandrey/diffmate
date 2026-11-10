use super::checks::{Anchored, Notes, Unanchored};
use crate::error::Error;
use crate::gh::InlineComment;

pub(super) fn from_snapshot(
    snapshot: &str,
    publication_id: &str,
) -> Result<(String, Vec<InlineComment>), Error> {
    let (loose, anchored, notes) = super::checks::payload(snapshot)?;
    let comments = anchored.into_iter().map(comment).collect();
    Ok((text(&loose, &notes, publication_id), comments))
}

pub(super) fn inline(id: &str, title: &str, body: &str) -> String {
    format!("{title}\n\n{body}\n\n<!-- diffmate:finding={id} -->")
}

fn comment(finding: Anchored) -> InlineComment {
    InlineComment {
        path: finding.path,
        line: finding.line,
        side: finding.side,
        body: inline(&finding.id, &finding.title, &finding.body),
    }
}

fn text(findings: &[Unanchored], notes: &Notes, publication_id: &str) -> String {
    let mut parts = Vec::new();
    for finding in findings {
        parts.push(section(finding));
    }
    if let Some(limitation) = &notes.limitation {
        parts.push(format!("Limitation: {limitation}"));
    }
    if !notes.future.is_empty() {
        let mut section = String::from("Future improvements");
        for item in &notes.future {
            section.push('\n');
            section.push_str("- ");
            section.push_str(item);
        }
        parts.push(section);
    }
    parts.push(format!("<!-- diffmate:publication={publication_id} -->"));
    parts.join("\n\n")
}

fn section(finding: &Unanchored) -> String {
    match &finding.path {
        Some(path) => format!("{path}\n\n{}\n\n{}", finding.title, finding.body),
        None => format!("{}\n\n{}", finding.title, finding.body),
    }
}
