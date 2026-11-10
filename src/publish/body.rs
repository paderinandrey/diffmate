use super::checks::{Notes, Unanchored};
use super::load::comments;
use crate::error::Error;
use crate::gh::InlineComment;
use crate::store::Store;

pub(super) async fn parts(
    store: &Store,
    draft_id: i64,
    publication_id: &str,
) -> Result<(String, Vec<InlineComment>), Error> {
    let notes = super::checks::notes(store, draft_id).await?;
    let loose = super::checks::unanchored(store, draft_id).await?;
    let inline = comments(store, draft_id).await?;
    Ok((text(&loose, &notes, publication_id), inline))
}

pub(super) fn inline(id: &str, title: &str, body: &str) -> String {
    format!("{title}\n\n{body}\n\n<!-- diffmate:finding={id} -->")
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
