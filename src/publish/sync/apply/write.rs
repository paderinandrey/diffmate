use super::{Link, missing};
use crate::gh::ThreadState;
use rusqlite::{Transaction, params};

pub(super) fn links(tx: &Transaction, links: &[Link]) -> rusqlite::Result<()> {
    let sql = "UPDATE findings SET github_comment_id = ?2, github_thread_id = ?3 \
               WHERE id = ?1 AND line IS NOT NULL";
    for link in links {
        tx.execute(sql, params![link.finding, link.comment, link.thread])?;
    }
    Ok(())
}

pub(super) fn statuses(tx: &Transaction, statuses: Option<&[ThreadState]>) -> rusqlite::Result<()> {
    let Some(statuses) = statuses else {
        return Ok(());
    };
    let sql =
        "UPDATE findings SET thread_status = ?2 WHERE github_thread_id = ?1 AND line IS NOT NULL";
    for status in statuses {
        let name = if status.resolved {
            "resolved"
        } else {
            "unresolved"
        };
        tx.execute(sql, params![status.thread_id, name])?;
    }
    Ok(())
}

pub(super) fn open_inline(tx: &Transaction, publication_id: &str) -> rusqlite::Result<i64> {
    let snapshot: String = tx.query_row(
        "SELECT snapshot FROM publications WHERE id = ?1",
        params![publication_id],
        |row| row.get(0),
    )?;
    let ids = super::anchored_ids(&snapshot).map_err(super::sql_json)?;
    let sql = "SELECT COUNT(*) FROM findings WHERE id = ?1 AND line IS NOT NULL \
               AND (github_comment_id IS NULL OR thread_status = 'unknown')";
    let mut open = 0_i64;
    for id in ids {
        open += tx.query_row(sql, params![id], |row| row.get::<_, i64>(0))?;
    }
    Ok(open)
}

pub(super) fn finish(tx: &Transaction, job_id: i64, now: i64, open: i64) -> rusqlite::Result<()> {
    let changed = if open == 0 {
        let sql = "UPDATE jobs SET state = 'succeeded', next_attempt_at = NULL, updated_at = ?2 \
                   WHERE id = ?1 AND kind = 'sync'";
        tx.execute(sql, params![job_id, now])?
    } else {
        let sql = "UPDATE jobs SET state = 'queued', next_attempt_at = ?2, updated_at = ?3 \
                   WHERE id = ?1 AND kind = 'sync'";
        tx.execute(sql, params![job_id, now + super::super::DELAY_MS, now])?
    };
    if changed == 1 { Ok(()) } else { Err(missing()) }
}
