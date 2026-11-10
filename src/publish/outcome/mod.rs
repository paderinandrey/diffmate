mod settle;

use std::time::Duration;

use super::checks::transient_gh;
use crate::error::Error;
use crate::gh::{Gh, ReviewRequest};
use crate::pr_url::PrRef;
use crate::store::Store;

const LIMIT: Duration = Duration::from_secs(30);

pub(super) async fn finish(
    store: &Store,
    gh: &impl Gh,
    pr: &PrRef,
    request: &ReviewRequest,
    job_id: i64,
    publication_id: &str,
) -> Result<(), Error> {
    let posted = tokio::time::timeout(LIMIT, gh.create_review(pr, request)).await;
    match posted {
        Ok(Ok(created)) => submitted(store, job_id, publication_id, &created.id).await,
        Ok(Err(error)) if transient_gh(&error) => unknown(store, job_id, publication_id).await,
        Ok(Err(_)) => rejected(store, job_id, publication_id).await,
        Err(_) => unknown(store, job_id, publication_id).await,
    }
}

async fn submitted(
    store: &Store,
    job_id: i64,
    publication_id: &str,
    review_id: &str,
) -> Result<(), Error> {
    settle::settle(
        store,
        job_id,
        publication_id,
        "submitted",
        Some(review_id),
        None,
    )
    .await
}

async fn unknown(store: &Store, job_id: i64, publication_id: &str) -> Result<(), Error> {
    settle::settle(
        store,
        job_id,
        publication_id,
        "unknown",
        None,
        Some("outcome unknown"),
    )
    .await
}

async fn rejected(store: &Store, job_id: i64, publication_id: &str) -> Result<(), Error> {
    settle::settle(
        store,
        job_id,
        publication_id,
        "rejected",
        None,
        Some("rejected"),
    )
    .await
}
