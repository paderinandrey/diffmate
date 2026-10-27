mod raw;
#[cfg(test)]
mod tests;

use super::GhCli;
use crate::error::Error;
use crate::pr_url::PrRef;
use crate::proc::run_tool;

const QUERY: &str = concat!(
    "query($owner:String!,$name:String!,$number:Int!){repository(owner:$owner,name:$name){",
    "pullRequest(number:$number){",
    "comments(first:100){pageInfo{hasNextPage}nodes{author{login}body createdAt}}",
    "reviews(first:50){pageInfo{hasNextPage}nodes{author{login}state body submittedAt}}",
    "reviewThreads(first:100){pageInfo{hasNextPage}nodes{isResolved isOutdated path line ",
    "comments(first:50){pageInfo{hasNextPage}nodes{author{login}body createdAt}}}}",
    "}}}"
);

pub(crate) struct Comment {
    pub author: String,
    pub body: String,
    pub created_at: String,
}

pub(crate) struct Review {
    pub author: String,
    pub state: String,
    pub body: String,
    pub submitted_at: String,
}

pub(crate) struct Thread {
    pub resolved: bool,
    pub outdated: bool,
    pub path: Option<String>,
    pub line: Option<u32>,
    pub comments: Vec<Comment>,
}

pub(crate) struct Discussion {
    pub comments: Vec<Comment>,
    pub reviews: Vec<Review>,
    pub threads: Vec<Thread>,
    pub truncated: bool,
}

pub(super) async fn fetch(gh: &GhCli, pr: &PrRef) -> Result<Discussion, Error> {
    let args = [
        "api".to_string(),
        "graphql".into(),
        "-f".into(),
        format!("query={QUERY}"),
        "-f".into(),
        format!("owner={}", pr.owner),
        "-f".into(),
        format!("name={}", pr.repo),
        "-F".into(),
        format!("number={}", pr.number),
    ];
    let stdout = Box::pin(run_tool("gh", gh.command(&args))).await?;
    raw::parse(&stdout)
}
