mod parse;

use super::GhCli;
use crate::error::Error;
use crate::pr_url::PrRef;
use crate::proc::run_tool;

pub(crate) use parse::{CommentLink, ThreadState};

const COMMENTS: &str = "query($owner:String!,$name:String!,$number:Int!){repository(owner:$owner,name:$name){pullRequest(number:$number){reviewThreads(first:100){nodes{id comments(first:100){nodes{databaseId body pullRequestReview{databaseId}}}}}}}}";

const STATUSES: &str = "query($owner:String!,$name:String!,$number:Int!){repository(owner:$owner,name:$name){pullRequest(number:$number){reviewThreads(first:100){nodes{id isResolved}}}}}";

pub(super) async fn comments(
    gh: &GhCli,
    pr: &PrRef,
    review_id: &str,
) -> Result<Vec<CommentLink>, Error> {
    parse::comments(&query(gh, pr, COMMENTS).await?, review_id)
}

pub(super) async fn statuses(gh: &GhCli, pr: &PrRef) -> Result<Vec<ThreadState>, Error> {
    parse::statuses(&query(gh, pr, STATUSES).await?)
}

async fn query(gh: &GhCli, pr: &PrRef, query: &str) -> Result<Vec<u8>, Error> {
    let args = [
        "api".to_string(),
        "graphql".into(),
        "-f".into(),
        format!("query={query}"),
        "-f".into(),
        format!("owner={}", pr.owner),
        "-f".into(),
        format!("name={}", pr.repo),
        "-F".into(),
        format!("number={}", pr.number),
    ];
    Box::pin(run_tool("gh", gh.command(&args))).await
}
