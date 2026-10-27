use super::{JobId, RunId, Store, now_ms};
use crate::error::Error;
use rusqlite::{OptionalExtension, Transaction, TransactionBehavior};
use std::collections::HashMap;

pub(crate) struct Stopped {
    pub runs: Vec<RunId>,
    pub jobs: Vec<JobId>,
}

pub(crate) struct RunLocation {
    pub owner: String,
    pub repo: String,
    pub running: bool,
}

fn ids(tx: &Transaction, sql: &str) -> rusqlite::Result<Vec<i64>> {
    let mut statement = tx.prepare(sql)?;
    statement.query_map([], |row| row.get(0))?.collect()
}

impl Store {
    pub async fn stop_running(&self) -> Result<Stopped, Error> {
        self.call(|conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let runs = ids(
                &tx,
                "SELECT id FROM runs WHERE state = 'running' ORDER BY id",
            )?;
            let jobs =
                "SELECT id FROM jobs WHERE state = 'running' AND kind = 'review' ORDER BY id";
            let jobs = ids(&tx, jobs)?;
            let now = now_ms();
            tx.execute(
                "UPDATE runs SET state = 'failed', failure_kind = 'interrupted',
                 interrupted_cause = 'crash', finished_at = ?1 WHERE state = 'running'",
                [now],
            )?;
            tx.execute(
                "UPDATE jobs SET state = 'failed', failure_reason = 'interrupted: crash',
                 updated_at = ?1 WHERE state = 'running' AND kind = 'review'",
                [now],
            )?;
            tx.commit()?;
            Ok(Stopped { runs, jobs })
        })
        .await
    }

    pub async fn run_locations(
        &self,
        runs: Vec<RunId>,
    ) -> Result<HashMap<RunId, RunLocation>, Error> {
        self.call(move |conn| {
            let sql = "SELECT t.owner, t.repo, r.state = 'running' FROM runs r
                       JOIN targets t ON t.id = r.target_id WHERE r.id = ?1";
            let mut statement = conn.prepare(sql)?;
            let mut found = HashMap::new();
            for run in runs {
                let row = statement.query_row([run], |row| {
                    Ok(RunLocation {
                        owner: row.get(0)?,
                        repo: row.get(1)?,
                        running: row.get(2)?,
                    })
                });
                if let Some(location) = row.optional()? {
                    found.insert(run, location);
                }
            }
            Ok(found)
        })
        .await
    }
}
