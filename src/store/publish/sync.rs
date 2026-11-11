use rusqlite::{Transaction, params};

pub(crate) fn insert(tx: &Transaction, publication_id: &str, now: i64) -> rusqlite::Result<()> {
    let pid = i64::from(std::process::id());
    let sql = "INSERT INTO jobs (kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, \
               created_at, updated_at, origin, publication_id) \
               SELECT 'sync', p.target_id, 'queued', ?2, ?3, ?3, ?3, 'manual', p.id \
               FROM publications p WHERE p.id = ?1 AND p.state = 'submitted' AND NOT EXISTS ( \
               SELECT 1 FROM jobs j WHERE j.publication_id = p.id AND j.kind = 'sync' \
               AND j.state IN ('queued', 'running'))";
    tx.execute(sql, params![publication_id, pid, now])?;
    Ok(())
}
