mod parse;

#[cfg(test)]
mod tests;

use crate::error::Error;
use std::future::Future;

const PAGE: u32 = 100;
const CAP: u32 = 1_000;

pub(super) fn search_args(limit: u32) -> [String; 8] {
    [
        "search".into(),
        "prs".into(),
        "--review-requested=@me".into(),
        "--state=open".into(),
        "--limit".into(),
        limit.to_string(),
        "--json".into(),
        "number,repository,url,isDraft,state".into(),
    ]
}

pub(super) async fn search_review_requested<T, F, Fut>(
    state: &mut T,
    mut fetch: F,
) -> Result<Vec<super::DirectRequest>, Error>
where
    F: FnMut(&mut T, u32) -> Fut + Send,
    Fut: Future<Output = Result<Vec<u8>, Error>> + Send,
{
    let mut limit = PAGE;
    loop {
        let page = parse::parse_page(&fetch(state, limit).await?)?;
        if !full(page.len, limit) || limit >= CAP {
            return Ok(page.kept);
        }
        limit += PAGE;
    }
}

fn full(len: usize, limit: u32) -> bool {
    u32::try_from(len).is_ok_and(|len| len == limit)
}
