use super::Outcome;
use crate::error::Error;
use crate::store::Store;
use rusqlite::{TransactionBehavior, params};

pub(super) async fn settle(
    store: &Store,
    job_id: i64,
    publication_id: &str,
    outcome: Outcome,
) -> Result<(), Error> {
    let publication_id = publication_id.to_string();
    let now = crate::store::now_ms();
    store
        .call(move |conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let (state, review_id, failure) = match &outcome {
                Outcome::Submitted(id) => ("submitted", Some(id.as_str()), None),
                Outcome::Rejected(why) => ("rejected", None, Some(why.as_str())),
                Outcome::Unknown(why) => ("unknown", None, Some(why.as_ref())),
            };
            let sql = "UPDATE publications SET state = ?2, github_review_id = ?3, reason = ?4, updated_at = ?5 WHERE id = ?1 AND state = 'sending'";
            let changed = tx.execute(sql, params![publication_id, state, review_id, failure, now])?;
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
            if state == "submitted" {
                crate::store::publish::insert_sync_job(&tx, &publication_id, now)?;
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
