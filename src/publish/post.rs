use super::body::from_snapshot;
use super::load::load_job;
use crate::error::Error;
use crate::gh::{Gh, ReviewRequest};
use crate::pr_url::PrRef;
use crate::store::Store;
use rusqlite::{TransactionBehavior, params};

pub(crate) async fn execute_publish(
    store: &Store,
    gh: &impl Gh,
    job_id: i64,
    max_retries: u32,
) -> Result<(), Error> {
    let job = load_job(store, job_id).await?;
    if !super::gate::before_send(store, gh, &job, job_id, max_retries).await? {
        return Ok(());
    }
    mark_sending(store, &job.publication_id).await?;
    let (body, comments) = from_snapshot(&job.snapshot, &job.publication_id)?;
    let request = ReviewRequest {
        commit_id: job.head_sha,
        body,
        event: job.event,
        comments,
    };
    let pr = PrRef {
        owner: job.owner,
        repo: job.repo,
        number: job.number,
    };
    let created = gh.create_review(&pr, &request).await?;
    mark_submitted(store, job_id, &job.publication_id, &created.id).await
}

async fn mark_sending(store: &Store, publication_id: &str) -> Result<(), Error> {
    let publication_id = publication_id.to_string();
    let now = crate::store::now_ms();
    let changed = store
        .call(move |conn| {
            let sql = "UPDATE publications SET state = 'sending', updated_at = ?2 WHERE id = ?1 AND state = 'pending'";
            conn.execute(sql, params![publication_id, now])
        })
        .await?;
    if changed == 0 {
        return Err(stuck("publication is not pending"));
    }
    Ok(())
}

async fn mark_submitted(
    store: &Store,
    job_id: i64,
    publication_id: &str,
    review_id: &str,
) -> Result<(), Error> {
    let publication_id = publication_id.to_string();
    let review_id = review_id.to_string();
    let now = crate::store::now_ms();
    store
        .call(move |conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let sql = "UPDATE publications SET state = 'submitted', github_review_id = ?2, updated_at = ?3 WHERE id = ?1 AND state = 'sending'";
            if tx.execute(sql, params![publication_id, review_id, now])? != 1 {
                return Err(constraint("publication is not sending"));
            }
            let sql = "UPDATE jobs SET state = 'succeeded', updated_at = ?2 WHERE id = ?1 AND kind = 'publish'";
            if tx.execute(sql, params![job_id, now])? != 1 {
                return Err(constraint("publish job is missing"));
            }
            tx.commit()?;
            Ok(())
        })
        .await
}

fn stuck(detail: &str) -> Error {
    Error::Command {
        program: "gh",
        args: "publish".into(),
        detail: detail.into(),
    }
}

fn constraint(detail: &str) -> rusqlite::Error {
    rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
        Some(detail.into()),
    )
}
