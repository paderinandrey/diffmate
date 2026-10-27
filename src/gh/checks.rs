use super::GhCli;
use crate::error::Error;
use crate::pr_url::PrRef;
use crate::proc::run_tool;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Output {
    pub title: Option<String>,
    pub summary: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct CheckRun {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub output: Option<Output>,
}

#[derive(Deserialize)]
pub(crate) struct CheckRuns {
    pub total_count: u64,
    #[serde(rename = "check_runs")]
    pub runs: Vec<CheckRun>,
}

#[derive(Deserialize)]
pub(crate) struct Annotation {
    pub path: String,
    pub start_line: u32,
    #[serde(rename = "annotation_level")]
    pub level: Option<String>,
    pub message: Option<String>,
}

async fn get(gh: &GhCli, endpoint: String) -> Result<Vec<u8>, Error> {
    Box::pin(run_tool("gh", gh.command(["api", endpoint.as_str()]))).await
}

pub(super) async fn runs(gh: &GhCli, pr: &PrRef, sha: &str) -> Result<CheckRuns, Error> {
    if sha.is_empty() || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Error::Command {
            program: "gh",
            args: "api check-runs".into(),
            detail: "the head SHA is not hexadecimal".into(),
        });
    }
    let (owner, repo) = (&pr.owner, &pr.repo);
    let endpoint = format!("repos/{owner}/{repo}/commits/{sha}/check-runs?per_page=100");
    Ok(serde_json::from_slice(&get(gh, endpoint).await?)?)
}

pub(super) async fn annotations(gh: &GhCli, pr: &PrRef, id: u64) -> Result<Vec<Annotation>, Error> {
    let (owner, repo) = (&pr.owner, &pr.repo);
    let endpoint = format!("repos/{owner}/{repo}/check-runs/{id}/annotations?per_page=50");
    Ok(serde_json::from_slice(&get(gh, endpoint).await?)?)
}
