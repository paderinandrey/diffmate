mod parse;
#[cfg(test)]
mod tests;

use super::GhCli;
use crate::error::Error;
use crate::pr_url::PrRef;
use crate::proc::{run_tool, run_tool_with_stdin};
use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ListedReview {
    pub id: String,
    pub author: String,
    pub state: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InlineComment {
    pub path: String,
    pub line: u32,
    pub side: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewRequest {
    pub commit_id: String,
    pub body: String,
    pub event: String,
    pub comments: Vec<InlineComment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CreatedReview {
    pub id: String,
}

pub(super) fn endpoint(pr: &PrRef) -> String {
    format!("repos/{}/{}/pulls/{}/reviews", pr.owner, pr.repo, pr.number)
}

pub(super) fn post_parts(pr: &PrRef, review: &ReviewRequest) -> Result<(String, Vec<u8>), Error> {
    let comments: Vec<Value> = review
        .comments
        .iter()
        .map(|comment| {
            json!({
                "path": comment.path,
                "line": comment.line,
                "side": comment.side,
                "body": comment.body,
            })
        })
        .collect();
    let payload = json!({
        "commit_id": review.commit_id,
        "body": review.body,
        "event": review.event,
        "comments": comments,
    });
    Ok((endpoint(pr), serde_json::to_vec(&payload)?))
}

pub(super) async fn current_login(gh: &GhCli) -> Result<String, Error> {
    let stdout = Box::pin(run_tool("gh", gh.command(["api", "user"]))).await?;
    parse::login_from(&stdout)
}

pub(super) async fn list_my_reviews(gh: &GhCli, pr: &PrRef) -> Result<Vec<ListedReview>, Error> {
    let path = endpoint(pr);
    let stdout = Box::pin(run_tool(
        "gh",
        gh.command(["api", "--paginate", path.as_str()]),
    ))
    .await?;
    parse::reviews_from(&stdout)
}

pub(super) async fn create_review(
    gh: &GhCli,
    pr: &PrRef,
    review: &ReviewRequest,
) -> Result<CreatedReview, Error> {
    let (path, bytes) = post_parts(pr, review)?;
    let args = ["api", "--method", "POST", path.as_str(), "--input", "-"];
    let stdout = Box::pin(run_tool_with_stdin("gh", gh.command(args), bytes)).await?;
    parse::created_from(&stdout)
}
