mod write;

use crate::error::Error;
use crate::gh::{CommentLink, ThreadState};
use crate::store::Store;
use rusqlite::{Connection, TransactionBehavior, params};

pub(super) struct Link {
    finding: String,
    comment: String,
    thread: String,
}

pub(super) async fn matched(
    store: &Store,
    publication_id: &str,
    comments: &[CommentLink],
) -> Result<Vec<Link>, Error> {
    let publication_id = publication_id.to_string();
    let findings = store
        .call(move |conn| finding_ids(conn, &publication_id))
        .await?;
    Ok(findings
        .into_iter()
        .filter_map(|finding| link(&finding, comments))
        .collect())
}

pub(super) async fn store_links(
    store: &Store,
    job_id: i64,
    publication_id: &str,
    links: Vec<Link>,
    statuses: Option<Vec<ThreadState>>,
) -> Result<(), Error> {
    let publication_id = publication_id.to_string();
    store
        .call(move |conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            write::links(&tx, &links)?;
            write::statuses(&tx, statuses.as_deref())?;
            let open = write::open_inline(&tx, &publication_id)?;
            write::finish(&tx, job_id, crate::store::now_ms(), open)?;
            tx.commit()?;
            Ok(())
        })
        .await
}

pub(super) fn missing() -> rusqlite::Error {
    rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
        Some("sync job is missing".into()),
    )
}

fn finding_ids(conn: &Connection, publication_id: &str) -> rusqlite::Result<Vec<String>> {
    let sql = "SELECT f.id FROM findings f JOIN publications p ON p.draft_id = f.draft_id \
               WHERE p.id = ?1 AND f.decision IN ('kept', 'edited') AND f.line IS NOT NULL";
    let mut statement = conn.prepare(sql)?;
    let rows = statement.query_map(params![publication_id], |row| row.get(0))?;
    rows.collect()
}

fn link(finding: &str, comments: &[CommentLink]) -> Option<Link> {
    let marker = format!("<!-- diffmate:finding={finding} -->");
    let comment = comments.iter().find(|item| item.body.contains(&marker))?;
    Some(Link {
        finding: finding.to_string(),
        comment: comment.comment_id.clone(),
        thread: comment.thread_id.clone(),
    })
}
