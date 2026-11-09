use super::super::DirectRequest;
use crate::error::Error;
use crate::pr_url::{self, PrRef};
use serde::Deserialize;

pub(super) struct Page {
    pub kept: Vec<DirectRequest>,
    pub len: usize,
}

#[derive(Deserialize)]
struct Repo {
    #[serde(rename = "nameWithOwner")]
    name_with_owner: String,
}

#[derive(Deserialize)]
struct Hit {
    number: u64,
    repository: Repo,
    url: String,
    #[serde(rename = "isDraft")]
    is_draft: bool,
    state: String,
}

pub(super) fn parse_page(bytes: &[u8]) -> Result<Page, Error> {
    let values: Vec<serde_json::Value> = serde_json::from_slice(bytes)?;
    let len = values.len();
    let kept = values.iter().filter_map(keep).collect();
    Ok(Page { kept, len })
}

fn keep(value: &serde_json::Value) -> Option<DirectRequest> {
    let hit = Hit::deserialize(value).ok()?;
    let pr = pr_url::parse(&hit.url).ok()?;
    let open = hit.state.eq_ignore_ascii_case("open");
    let same = same_repo(&hit.repository.name_with_owner, &pr);
    (hit.number == pr.number && same && open).then_some(DirectRequest {
        pr,
        is_draft: hit.is_draft,
    })
}

fn same_repo(name: &str, pr: &PrRef) -> bool {
    name.eq_ignore_ascii_case(&format!("{}/{}", pr.owner, pr.repo))
}
