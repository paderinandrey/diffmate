mod parse;

use super::GhCli;
use crate::error::Error;
use crate::pr_url::PrRef;
use crate::proc::run_tool;

pub(crate) use parse::{CommentLink, ThreadState};

const COMMENTS: &str = "query($owner:String!,$name:String!,$number:Int!,$cursor:String){repository(owner:$owner,name:$name){pullRequest(number:$number){reviewThreads(first:100,after:$cursor){pageInfo{hasNextPage endCursor}nodes{id comments(first:100){nodes{databaseId body pullRequestReview{databaseId}}}}}}}}";

const STATUSES: &str = "query($owner:String!,$name:String!,$number:Int!,$cursor:String){repository(owner:$owner,name:$name){pullRequest(number:$number){reviewThreads(first:100,after:$cursor){pageInfo{hasNextPage endCursor}nodes{id isResolved}}}}}";

pub(super) async fn comments(
    gh: &GhCli,
    pr: &PrRef,
    review_id: &str,
) -> Result<Vec<CommentLink>, Error> {
    let mut links = Vec::new();
    for page in pages(gh, pr, COMMENTS).await? {
        links.extend(parse::comments(&page, review_id)?);
    }
    Ok(links)
}

pub(super) async fn statuses(gh: &GhCli, pr: &PrRef) -> Result<Vec<ThreadState>, Error> {
    let mut rows = Vec::new();
    for page in pages(gh, pr, STATUSES).await? {
        rows.extend(parse::statuses(&page)?);
    }
    Ok(rows)
}

async fn pages(gh: &GhCli, pr: &PrRef, query: &str) -> Result<Vec<Vec<u8>>, Error> {
    let mut cursor = None;
    let mut found = Vec::new();
    loop {
        let page = request(gh, pr, query, cursor.as_deref()).await?;
        let next = parse::page_after(&page)?;
        found.push(page);
        let Some(next) = next else {
            return Ok(found);
        };
        if cursor.as_deref() == Some(next.as_str()) {
            return Err(Error::Command {
                program: "gh",
                args: "api graphql".into(),
                detail: "reviewThreads cursor did not advance".into(),
            });
        }
        cursor = Some(next);
    }
}

async fn request(
    gh: &GhCli,
    pr: &PrRef,
    query: &str,
    cursor: Option<&str>,
) -> Result<Vec<u8>, Error> {
    let mut args = vec![
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
    if let Some(cursor) = cursor {
        args.push("-f".into());
        args.push(format!("cursor={cursor}"));
    }
    Box::pin(run_tool("gh", gh.command(&args))).await
}
