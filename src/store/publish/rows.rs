use rusqlite::Transaction;

pub(super) struct Draft {
    pub run_id: i64,
    pub target_id: i64,
    pub limitation: Option<String>,
    pub included: bool,
    pub future: String,
}

pub(super) struct Finding {
    pub id: String,
    pub decision: String,
    pub severity: String,
    pub path: Option<String>,
    pub line: Option<i64>,
    pub side: Option<String>,
    pub title: String,
    pub body: String,
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
            future: row.get(4)?,
        })
    })
}

pub(super) fn findings(tx: &Transaction, draft_id: i64) -> rusqlite::Result<Vec<Finding>> {
    let sql = "SELECT id, decision, severity, path, line, side, title, body FROM findings WHERE draft_id = ?1 ORDER BY (line IS NULL OR path IS NULL), ordinal";
    let mut stmt = tx.prepare(sql)?;
    let rows = stmt.query_map([draft_id], |row| {
        Ok(Finding {
            id: row.get(0)?,
            decision: row.get(1)?,
            severity: row.get(2)?,
            path: row.get(3)?,
            line: row.get(4)?,
            side: row.get(5)?,
            title: row.get(6)?,
            body: row.get(7)?,
        })
    })?;
    rows.collect()
}

pub(super) fn undecided(findings: &[Finding]) -> Vec<String> {
    findings
        .iter()
        .filter(|finding| finding.decision == "undecided")
        .map(|finding| finding.id.clone())
        .collect()
}
