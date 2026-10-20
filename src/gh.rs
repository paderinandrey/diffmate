use crate::error::Error;
use crate::pr_url::PrRef;
use crate::proc::run_tool;
use crate::procenv::{InheritedEnv, Pairs};
use serde::Deserialize;
use std::future::Future;
use tokio::process::Command;

const FIELDS: &str =
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
struct Author {
    login: String,
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

pub(crate) trait Gh {
    fn pr_view(&self, pr: &PrRef) -> impl Future<Output = Result<PrMeta, Error>> + Send;
}

pub(crate) struct GhCli {
    env: Pairs,
}

impl GhCli {
    pub fn new(env: &InheritedEnv) -> GhCli {
        let mut vars = env.tool_env();
        for (name, value) in [
            ("GH_PROMPT_DISABLED", "1"),
            ("GH_NO_UPDATE_NOTIFIER", "1"),
            ("NO_COLOR", "1"),
        ] {
            vars.push((name.into(), value.into()));
        }
        GhCli { env: vars }
    }
}

impl Gh for GhCli {
    async fn pr_view(&self, pr: &PrRef) -> Result<PrMeta, Error> {
        let mut command = Command::new("gh");
        command
            .args(["pr", "view"])
            .arg(pr.web_url())
            .args(["--json", FIELDS]);
        command
            .env_clear()
            .envs(self.env.iter().map(|(name, value)| (name, value)));
        let stdout = Box::pin(run_tool("gh", command)).await?;
        Ok(serde_json::from_slice::<Raw>(&stdout)?.into())
    }
}
