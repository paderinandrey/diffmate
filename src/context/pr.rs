use super::caps::{PR_BODY_CAP, cap_split};
use crate::gh::PrMeta;
use crate::pr_url::PrRef;
use crate::store::records::Revision;

pub(super) fn render(pr: &PrRef, meta: &PrMeta, revision: &Revision) -> String {
    let fork = if meta.is_cross_repository {
        "yes"
    } else {
        "no"
    };
    let total = meta.body.len();
    let (marker, body) = cap_split("pr.md description", &meta.body, total, PR_BODY_CAP);
    let marker = marker.map(|line| format!("{line}\n")).unwrap_or_default();
    format!(
        "{marker}# {label} (#{number})\n\nTitle: {title}\nAuthor: {author}\nState: {state}\nURL: {url}\nBase branch: {base}\nHead branch: {head}\nHead SHA: {head_sha}\nMerge-base SHA: {merge_base}\nHead is in another repository: {fork}\n\n## Description\n\n{body}\n",
        label = pr.label(),
        number = meta.number,
        title = meta.title,
        author = meta.author,
        state = meta.state,
        url = meta.url,
        base = meta.base_ref,
        head = meta.head_ref,
        head_sha = revision.head_sha,
        merge_base = revision.merge_base_sha,
    )
}
