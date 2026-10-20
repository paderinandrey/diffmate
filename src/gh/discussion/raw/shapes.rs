use crate::gh::meta::Author;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PageInfo {
    pub has_next_page: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Page<T> {
    pub page_info: PageInfo,
    pub nodes: Vec<Option<T>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawComment {
    pub author: Option<Author>,
    pub body: String,
    pub created_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawReview {
    pub author: Option<Author>,
    pub state: String,
    pub body: String,
    pub submitted_at: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawThread {
    pub is_resolved: bool,
    pub is_outdated: bool,
    pub path: Option<String>,
    pub line: Option<u32>,
    pub comments: Page<RawComment>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Pull {
    pub comments: Page<RawComment>,
    pub reviews: Page<RawReview>,
    pub review_threads: Page<RawThread>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Repository {
    pub pull_request: Option<Pull>,
}

#[derive(Deserialize)]
pub(super) struct Data {
    pub repository: Option<Repository>,
}

#[derive(Deserialize)]
pub(super) struct Envelope {
    pub data: Data,
}
