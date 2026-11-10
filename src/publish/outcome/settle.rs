use crate::error::Error;
use crate::store::Store;
use rusqlite::{TransactionBehavior, params};

pub(super) async fn settle(
    store: &Store,
    job_id: i64,
    publication_id: &str,
    state: &'static str,
    review_id: Option<&str>,
    failure: Option<&'static str>,
) -> Result<(), Error> {
    let publication_id = publication_id.to_string();
    let review_id = review_id.map(str::to_string);
    let now = crate::store::now_ms();
    store
        .call(move |conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let changed = if let Some(review_id) = &review_id {
                let sql = "UPDATE publications SET state = ?2, github_review_id = ?3, updated_at = ?4 WHERE id = ?1 AND state = 'sending'";
                tx.execute(sql, params![publication_id, state, review_id, now])?
            } else {
                let sql = "UPDATE publications SET state = ?2, updated_at = ?3 WHERE id = ?1 AND state = 'sending'";
                tx.execute(sql, params![publication_id, state, now])?
            };
            if changed != 1 {
                return Err(constraint("publication is not sending"));
            }
            let changed = if let Some(reason) = failure {
                let sql = "UPDATE jobs SET state = 'failed', failure_reason = ?2, updated_at = ?3 WHERE id = ?1 AND kind = 'publish'";
                tx.execute(sql, params![job_id, reason, now])?
            } else {
                let sql = "UPDATE jobs SET state = 'succeeded', updated_at = ?2 WHERE id = ?1 AND kind = 'publish'";
                tx.execute(sql, params![job_id, now])?
            };
            if changed != 1 {
                return Err(constraint("publish job is missing"));
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
