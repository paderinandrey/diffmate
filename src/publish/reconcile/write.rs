use crate::error::Error;
use crate::store::Store;
use rusqlite::{TransactionBehavior, params};

pub(super) async fn succeed(store: &Store, job_id: i64) -> Result<(), Error> {
    let now = crate::store::now_ms();
    store
        .call(move |conn| {
            let sql = "UPDATE jobs SET state = 'succeeded', updated_at = ?2 WHERE id = ?1 AND kind = 'reconcile'";
            if conn.execute(sql, params![job_id, now])? != 1 {
                return Err(constraint("reconcile job is missing"));
            }
            Ok(())
        })
        .await
}

pub(super) async fn miss(store: &Store, job_id: i64, error: &Error) -> Result<(), Error> {
    let reason = error.to_string();
    let now = crate::store::now_ms();
    store
        .call(move |conn| {
            let sql = "UPDATE jobs SET state = 'failed', failure_reason = ?2, updated_at = ?3 WHERE id = ?1 AND kind = 'reconcile'";
            if conn.execute(sql, params![job_id, reason, now])? != 1 {
                return Err(constraint("reconcile job is missing"));
            }
            Ok(())
        })
        .await
}

pub(super) async fn submit(
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
            let sql = "UPDATE publications SET state = 'submitted', github_review_id = ?2, updated_at = ?3 WHERE id = ?1 AND state = 'unknown'";
            tx.execute(sql, params![publication_id, review_id, now])?;
            let sql = "UPDATE jobs SET state = 'succeeded', updated_at = ?2 WHERE id = ?1 AND kind = 'reconcile'";
            if tx.execute(sql, params![job_id, now])? != 1 {
                return Err(constraint("reconcile job is missing"));
            }
            tx.commit()?;
            Ok(())
        })
        .await
}

fn constraint(detail: &str) -> rusqlite::Error {
    rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
        Some(detail.into()),
    )
}
