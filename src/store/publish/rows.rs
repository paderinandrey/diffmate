use crate::store::status::{DraftFinding, improvements};
use rusqlite::Transaction;

pub(super) struct Draft {
    pub run_id: i64,
    pub target_id: i64,
    pub limitation: Option<String>,
    pub included: bool,
    pub future: Vec<String>,
}

pub(super) fn draft(tx: &Transaction, draft_id: i64) -> rusqlite::Result<Draft> {
    let sql = "SELECT r.id, r.target_id, d.limitation, d.limitation_included, d.future_improvements FROM drafts d JOIN runs r ON r.id = d.run_id WHERE d.id = ?1";
    tx.query_row(sql, [draft_id], |row| {
        let included: i64 = row.get(3)?;
        Ok(Draft {
            run_id: row.get(0)?,
            target_id: row.get(1)?,
            limitation: row.get(2)?,
            included: included == 1,
            future: improvements(&row.get::<_, String>(4)?)?,
        })
    })
}

pub(super) fn undecided(findings: &[DraftFinding]) -> Vec<String> {
    findings
        .iter()
        .filter(|finding| finding.decision == "undecided")
        .map(|finding| finding.id.clone())
        .collect()
}
