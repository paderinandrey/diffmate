use crate::error::Error;
use serde::Deserialize;

pub(super) const FIELDS: &str =
    "number,url,title,body,author,baseRefName,headRefName,headRefOid,isCrossRepository,state";

pub(crate) struct PrMeta {
    pub number: u64,
    pub url: String,
    pub title: String,
    pub body: String,
    pub author: String,
    pub base_ref: String,
    pub head_ref: String,
    pub head_sha: String,
    pub is_cross_repository: bool,
    pub state: String,
}

#[derive(Deserialize)]
pub(super) struct Author {
    pub login: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Raw {
    number: u64,
    url: String,
    title: String,
    body: Option<String>,
    author: Option<Author>,
    base_ref_name: String,
    head_ref_name: String,
    head_ref_oid: String,
    is_cross_repository: bool,
    state: String,
}

impl From<Raw> for PrMeta {
    fn from(raw: Raw) -> PrMeta {
        PrMeta {
            number: raw.number,
            url: raw.url,
            title: raw.title,
            body: raw.body.unwrap_or_default(),
            author: raw.author.map_or_else(|| "unknown".into(), |a| a.login),
            base_ref: raw.base_ref_name,
            head_ref: raw.head_ref_name,
            head_sha: raw.head_ref_oid,
            is_cross_repository: raw.is_cross_repository,
            state: raw.state,
        }
    }
}

pub(super) fn parse(stdout: &[u8]) -> Result<PrMeta, Error> {
    Ok(serde_json::from_slice::<Raw>(stdout)?.into())
}
