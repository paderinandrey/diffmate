mod resolve;

use super::records::JobOrigin;
use super::retry::apply_job_next;
use super::{JobId, RunId, Store};
use crate::error::Error;
use crate::recovery::rules::{Failure, after_failure};
use rusqlite::{OptionalExtension, TransactionBehavior};
use std::collections::HashMap;

pub(crate) struct Stopped {
    pub runs: Vec<(RunId, String)>,
    pub jobs: Vec<(JobId, String)>,
    pub manual_crashes: Vec<JobId>,
}

pub(crate) struct RunLocation {
    pub owner: String,
    pub repo: String,
    pub running: bool,
}

impl Store {
    pub async fn resolve_running(&self, config_max: u32, now: i64) -> Result<Stopped, Error> {
        self.call(move |conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let mut stopped = Stopped {
                runs: resolve::runs(&tx, now)?,
                jobs: Vec::new(),
                manual_crashes: Vec::new(),
            };
            for job in resolve::jobs(&tx)? {
                let failure = Failure::from_stop(job.stop);
                if job.origin == JobOrigin::Manual && matches!(failure, Failure::Crash) {
                    stopped.manual_crashes.push(job.id);
                }
                let next = after_failure(job.origin, &failure, job.retry, config_max, now);
                apply_job_next(&tx, job.id, &next, now)?;
                stopped
                    .jobs
                    .push((job.id, resolve::description(&next, &failure)));
            }
            tx.commit()?;
            Ok(stopped)
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
