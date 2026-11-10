use super::{Outcome, rows, save};
use crate::store::now_ms;
use rusqlite::{Connection, Transaction, TransactionBehavior};

const SUBMITTED: &str =
    "SELECT EXISTS(SELECT 1 FROM publications WHERE draft_id = ?1 AND state = 'submitted')";
const BLOCKING: &str = "SELECT EXISTS(SELECT 1 FROM publications WHERE target_id = ?1 AND state IN ('pending', 'sending', 'unknown'))";

pub(super) fn write(
    conn: &mut Connection,
    draft_id: i64,
    event: &str,
    confirmed_sha: &str,
    pid: i64,
    started: i64,
) -> rusqlite::Result<Outcome> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let outcome = insert(&tx, draft_id, event, confirmed_sha, pid, started)?;
    if matches!(outcome, Outcome::Job(_)) {
        tx.commit()?;
    }
    Ok(outcome)
}

fn insert(
    tx: &Transaction,
    draft_id: i64,
    event: &str,
    confirmed_sha: &str,
    pid: i64,
    started: i64,
) -> rusqlite::Result<Outcome> {
    let draft = rows::draft(tx, draft_id)?;
    if flag(tx, SUBMITTED, draft_id)? {
        return Ok(Outcome::DraftSubmitted);
    }
    if flag(tx, BLOCKING, draft.target_id)? {
        return Ok(Outcome::Blocked);
    }
    let findings = rows::findings(tx, draft_id)?;
    let undecided = rows::undecided(&findings);
    if !undecided.is_empty() {
        return Ok(Outcome::Undecided(undecided));
    }
    let snapshot = save::snapshot(event, &draft, &findings)?;
    let now = now_ms();
    let publication_id =
        save::publication(tx, &draft, draft_id, event, confirmed_sha, &snapshot, now)?;
    save::job(tx, draft.target_id, pid, started, now, &publication_id)
}

fn flag(tx: &Transaction, sql: &str, id: i64) -> rusqlite::Result<bool> {
    tx.query_row(sql, [id], |row| row.get(0))
}
