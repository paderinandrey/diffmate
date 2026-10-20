use super::records::{DraftRecord, FindingRecord, Metrics};
use super::{JobId, RunId, Store, now_ms};
use crate::agent::Side;
use crate::error::Error;
use rusqlite::{Transaction, TransactionBehavior, params};

fn insert_finding(
    tx: &Transaction,
    ids: (RunId, i64, i64),
    f: &FindingRecord,
) -> rusqlite::Result<()> {
    let sql = "INSERT INTO findings (id, run_id, draft_id, ordinal, severity, path, line, side,
               title, body, previous_finding_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)";
    let (run, draft, n) = ids;
    let side = f.side.map(Side::as_str);
    let id = format!("{run}.{n}");
    let values = params![
        id,
        run,
        draft,
        n,
        f.severity.as_str(),
        f.path,
        f.line,
        side,
        f.title,
        f.body,
        f.previous_finding_id
    ];
    tx.execute(sql, values).map(drop)
}

impl Store {
    pub async fn finish_succeeded(
        &self,
        run: RunId,
        job: JobId,
        draft: DraftRecord,
        warnings: Vec<String>,
        metrics: Metrics,
    ) -> Result<(), Error> {
        let original = serde_json::to_string(&draft.original_output)?;
        let improvements = serde_json::to_string(&draft.future_improvements)?;
        let warnings = serde_json::to_string(&warnings)?;
        self.call(move |conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let result = if draft.findings.is_empty() {
                "no_findings"
            } else {
                "findings"
            };
            let now = now_ms();
            tx.execute(
                "UPDATE runs SET state = 'succeeded', result = ?1, total_cost_usd = ?2,
                 duration_ms = ?3, warnings = ?4, finished_at = ?5 WHERE id = ?6",
                params![
                    result,
                    metrics.total_cost_usd,
                    metrics.duration_ms,
                    warnings,
                    now,
                    run
                ],
            )?;
            tx.execute(
                "INSERT INTO drafts (run_id, original_output, limitation, future_improvements,
                 created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![run, original, draft.limitation, improvements, now],
            )?;
            let draft_id = tx.last_insert_rowid();
            for (n, finding) in (1_i64..).zip(&draft.findings) {
                insert_finding(&tx, (run, draft_id, n), finding)?;
            }
            let sql = "UPDATE jobs SET state = 'succeeded', updated_at = ?1 WHERE id = ?2";
            tx.execute(sql, params![now, job])?;
            tx.commit()
        })
        .await
    }
}
