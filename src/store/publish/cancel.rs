use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

pub(crate) enum Gate {
    Queued(String),
    Started,
    Other,
}

pub(crate) enum CancelQueued {
    Cancelled,
    Started,
    Absent,
}

struct Row {
    publication_id: String,
    job_id: i64,
    publication: String,
    job: String,
}

pub(super) fn gate(conn: &Connection, target: i64) -> rusqlite::Result<Gate> {
    let Some(row) = lookup(conn, target)? else {
        return Ok(Gate::Other);
    };
    Ok(classify(row))
}

pub(super) fn write(conn: &mut Connection, id: &str) -> rusqlite::Result<CancelQueued> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let sql = "SELECT p.id, j.id, p.state, j.state FROM publications p
               JOIN jobs j ON j.publication_id = p.id AND j.kind = 'publish'
               WHERE p.id = ?1";
    let Some(row) = tx.query_row(sql, [id], read_row).optional()? else {
        return Ok(CancelQueued::Absent);
    };
    if !pending(&row) {
        return Ok(if row.publication == "sending" || row.job == "running" {
            CancelQueued::Started
        } else {
            CancelQueued::Absent
        });
    }
    let now = crate::store::now_ms();
    let jobs = tx.execute(
        "UPDATE jobs SET state = 'cancelled', updated_at = ?2 WHERE id = ?1 AND state = 'queued'",
        params![row.job_id, now],
    )?;
    if jobs != 1 {
        return Err(constraint("publish job is not queued"));
    }
    let sql = "UPDATE publications SET state = 'abandoned', reason = 'cancelled by me', updated_at = ?2 WHERE id = ?1 AND state = 'pending'";
    if tx.execute(sql, params![row.publication_id, now])? != 1 {
        return Err(constraint("publication is not pending"));
    }
    tx.commit()?;
    Ok(CancelQueued::Cancelled)
}

fn classify(row: Row) -> Gate {
    if pending(&row) {
        Gate::Queued(row.publication_id)
    } else if row.publication == "sending" || row.job == "running" {
        Gate::Started
    } else {
        Gate::Other
    }
}

fn pending(row: &Row) -> bool {
    row.publication == "pending" && row.job == "queued"
}

fn lookup(conn: &Connection, target: i64) -> rusqlite::Result<Option<Row>> {
    let sql = "SELECT p.id, j.id, p.state, j.state FROM publications p \
               JOIN jobs j ON j.publication_id = p.id AND j.kind = 'publish' \
               WHERE p.target_id = ?1 AND p.state IN ('pending', 'sending') ORDER BY j.id DESC LIMIT 1";
    conn.query_row(sql, [target], read_row).optional()
}

fn read_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Row> {
    Ok(Row {
        publication_id: row.get(0)?,
        job_id: row.get(1)?,
        publication: row.get(2)?,
        job: row.get(3)?,
    })
}

fn constraint(detail: &str) -> rusqlite::Error {
    rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
        Some(detail.into()),
    )
}
