use rusqlite::{Connection, TransactionBehavior, params};

pub(super) fn needs_confirmation(
    conn: &Connection,
    job_id: i64,
    seen_sha: &str,
) -> rusqlite::Result<()> {
    let now = crate::store::now_ms();
    let sql = "UPDATE jobs SET state = 'needs_confirmation', head_sha = ?2, updated_at = ?3 WHERE id = ?1 AND kind = 'publish'";
    if conn.execute(sql, params![job_id, seen_sha, now])? != 1 {
        return Err(constraint("publish job is missing"));
    }
    Ok(())
}

pub(super) fn reconfirm(
    conn: &mut Connection,
    publication_id: &str,
    new_sha: &str,
) -> rusqlite::Result<bool> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let now = crate::store::now_ms();
    let sql = "UPDATE publications SET confirmed_sha = ?2, updated_at = ?3 WHERE id = ?1 AND state = 'pending'";
    if tx.execute(sql, params![publication_id, new_sha, now])? != 1 {
        return Ok(false);
    }
    let sql = "UPDATE jobs SET state = 'queued', updated_at = ?2 WHERE publication_id = ?1 AND kind = 'publish' AND state = 'needs_confirmation' AND head_sha = ?3";
    if tx.execute(sql, params![publication_id, now, new_sha])? != 1 {
        return Ok(false);
    }
    tx.commit()?;
    Ok(true)
}

pub(super) fn decline(
    conn: &mut Connection,
    publication_id: &str,
    seen_sha: &str,
) -> rusqlite::Result<bool> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let now = crate::store::now_ms();
    let sql = "UPDATE jobs SET state = 'cancelled', updated_at = ?2 WHERE publication_id = ?1 AND kind = 'publish' AND state = 'needs_confirmation' AND head_sha = ?3";
    if tx.execute(sql, params![publication_id, now, seen_sha])? != 1 {
        return Ok(false);
    }
    let sql = "UPDATE publications SET state = 'abandoned', reason = 'declined', updated_at = ?2 WHERE id = ?1 AND state = 'pending'";
    if tx.execute(sql, params![publication_id, now])? != 1 {
        return Ok(false);
    }
    tx.commit()?;
    Ok(true)
}

pub(super) fn abandon(
    conn: &mut Connection,
    publication_id: &str,
    job_id: i64,
    reason: &str,
) -> rusqlite::Result<()> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let now = crate::store::now_ms();
    let sql = "UPDATE publications SET state = 'abandoned', reason = ?2, updated_at = ?3 WHERE id = ?1 AND state = 'pending'";
    if tx.execute(sql, params![publication_id, reason, now])? != 1 {
        return Err(constraint("publication is not pending"));
    }
    let sql = "UPDATE jobs SET state = 'failed', failure_reason = ?2, updated_at = ?3 WHERE id = ?1 AND kind = 'publish'";
    if tx.execute(sql, params![job_id, reason, now])? != 1 {
        return Err(constraint("publish job is missing"));
    }
    tx.commit()?;
    Ok(())
}

fn constraint(detail: &str) -> rusqlite::Error {
    rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
        Some(detail.into()),
    )
}
