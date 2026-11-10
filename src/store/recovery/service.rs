mod write;

use crate::store::JobId;
use rusqlite::{Transaction, params};
use write::{fail, queue_unknown, requeue, succeed};

struct Running {
    id: JobId,
    kind: String,
    publication: Option<String>,
    state: Option<String>,
}

pub(super) fn settle(tx: &Transaction, now: i64) -> rusqlite::Result<Vec<(JobId, String)>> {
    let mut stopped = Vec::new();
    for job in running(tx)? {
        let text = match job.kind.as_str() {
            "publish" => settle_publish(tx, &job, now)?,
            "reconcile" | "sync" => {
                requeue(tx, job.id, now)?;
                "requeued".to_string()
            }
            _ => continue,
        };
        stopped.push((job.id, text));
    }
    queue_unknown(tx, now)?;
    Ok(stopped)
}

fn running(tx: &Transaction) -> rusqlite::Result<Vec<Running>> {
    let sql = "SELECT j.id, j.kind, j.publication_id, p.state FROM jobs j \
               LEFT JOIN publications p ON p.id = j.publication_id \
               WHERE j.state = 'running' AND j.kind IN ('publish', 'reconcile', 'sync') \
               ORDER BY j.id";
    let mut query = tx.prepare(sql)?;
    let rows = query.query_map([], |row| {
        Ok(Running {
            id: row.get(0)?,
            kind: row.get(1)?,
            publication: row.get(2)?,
            state: row.get(3)?,
        })
    })?;
    rows.collect()
}

fn settle_publish(tx: &Transaction, job: &Running, now: i64) -> rusqlite::Result<String> {
    match job.state.as_deref().unwrap_or("") {
        "pending" => {
            requeue(tx, job.id, now)?;
            Ok("requeued".to_string())
        }
        "sending" => {
            mark_unknown(tx, job, now)?;
            fail(tx, job.id, "outcome unknown", now)?;
            Ok("failed (outcome unknown)".to_string())
        }
        "unknown" => {
            fail(tx, job.id, "outcome unknown", now)?;
            Ok("failed (outcome unknown)".to_string())
        }
        "submitted" => {
            succeed(tx, job.id, now)?;
            Ok("succeeded".to_string())
        }
        other => {
            let reason = if other.is_empty() {
                "outcome unknown"
            } else {
                other
            };
            fail(tx, job.id, reason, now)?;
            Ok(format!("failed ({reason})"))
        }
    }
}

fn mark_unknown(tx: &Transaction, job: &Running, now: i64) -> rusqlite::Result<()> {
    let Some(id) = job.publication.as_deref() else {
        return Ok(());
    };
    let sql = "UPDATE publications SET state = 'unknown', updated_at = ?2 \
               WHERE id = ?1 AND state = 'sending'";
    write::changed(tx, sql, params![id, now], "publication is not sending")
}
