use super::{CreatedReview, ListedReview};
use crate::error::Error;
use serde::Deserialize;

#[derive(Deserialize)]
struct Login {
    login: String,
}

#[derive(Deserialize)]
struct RawReview {
    id: i64,
    user: Option<Login>,
    state: String,
    body: String,
}

#[derive(Deserialize)]
struct RawCreated {
    id: i64,
}

fn api_error(detail: &str) -> Error {
    Error::Command {
        program: "gh",
        args: "api".into(),
        detail: detail.into(),
    }
}

pub(super) fn login_from(bytes: &[u8]) -> Result<String, Error> {
    let user: Login = serde_json::from_slice(bytes)?;
    if user.login.is_empty() {
        return Err(api_error("github user has no login"));
    }
    Ok(user.login)
}

pub(super) fn reviews_from(bytes: &[u8]) -> Result<Vec<ListedReview>, Error> {
    let rows: Vec<RawReview> = serde_json::from_slice(bytes)?;
    rows.into_iter().map(one_review).collect()
}

fn one_review(row: RawReview) -> Result<ListedReview, Error> {
    let Some(author) = row
        .user
        .map(|user| user.login)
        .filter(|login| !login.is_empty())
    else {
        return Err(api_error("review has no author"));
    };
    Ok(ListedReview {
        id: row.id.to_string(),
        author,
        state: row.state,
        body: row.body,
    })
}

pub(super) fn created_from(bytes: &[u8]) -> Result<CreatedReview, Error> {
    let raw: RawCreated = serde_json::from_slice(bytes)?;
    Ok(CreatedReview {
        id: raw.id.to_string(),
    })
}
