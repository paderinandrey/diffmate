use super::{RunId, Store, TargetId};
use crate::error::Error;
use rusqlite::{OptionalExtension, params};

pub(crate) struct StoredFinding {
    pub id: String,
    pub severity: String,
    pub path: Option<String>,
    pub line: Option<u32>,
    pub side: Option<String>,
    pub title: String,
    pub body: String,
    pub decision: String,
}

pub(crate) struct PreviousRun {
    pub run_id: RunId,
    pub head_sha: String,
    pub findings: Vec<StoredFinding>,
}

impl Store {
    pub async fn latest_succeeded_before(
        &self,
        target: TargetId,
        run: RunId,
    ) -> Result<Option<PreviousRun>, Error> {
        self.call(move |conn| {
            let sql = "SELECT id, head_sha FROM runs
                       WHERE target_id = ?1 AND state = 'succeeded' AND id < ?2
                       ORDER BY id DESC LIMIT 1";
            let row = |r: &rusqlite::Row| Ok((r.get::<_, RunId>(0)?, r.get::<_, String>(1)?));
            let Some((run_id, head_sha)) =
                conn.query_row(sql, params![target, run], row).optional()?
            else {
                return Ok(None);
            };
            let sql = "SELECT id, severity, path, line, side, title, body, decision
                       FROM findings WHERE run_id = ?1 ORDER BY ordinal";
            let mut statement = conn.prepare(sql)?;
            let stored = |r: &rusqlite::Row| {
                Ok(StoredFinding {
                    id: r.get(0)?,
                    severity: r.get(1)?,
                    path: r.get(2)?,
                    line: r.get(3)?,
                    side: r.get(4)?,
                    title: r.get(5)?,
                    body: r.get(6)?,
                    decision: r.get(7)?,
                })
            };
            let findings = statement
                .query_map(params![run_id], stored)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Some(PreviousRun {
                run_id,
                head_sha,
                findings,
            }))
        })
        .await
    }
}
