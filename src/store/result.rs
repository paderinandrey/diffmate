mod read;

use super::records::FindingRecord;
use super::{JobId, RunId, Store, TargetId, now_ms};
use crate::agent::FailureKind;
use crate::error::Error;
use rusqlite::{OptionalExtension, params};

pub(crate) struct StoredJob {
    pub state: String,
    pub reason: Option<String>,
    pub run: Option<StoredRun>,
    pub reused: bool,
    pub known_head: Option<String>,
}

pub(crate) enum StoredRun {
    Succeeded {
        run_id: RunId,
        head_sha: String,
        findings: Vec<FindingRecord>,
        limitation: Option<String>,
        future_improvements: Vec<String>,
        warnings: Vec<String>,
    },
    Failed {
        run_id: RunId,
        kind: FailureKind,
        message: String,
    },
}

impl Store {
    pub async fn succeeded_run_for_head(
        &self,
        target: TargetId,
        head: &str,
    ) -> Result<Option<RunId>, Error> {
        let head = head.to_string();
        self.call(move |conn| {
            conn.query_row(
                "SELECT id FROM runs WHERE target_id = ?1 AND head_sha = ?2
                 AND state = 'succeeded' ORDER BY id DESC LIMIT 1",
                params![target, head],
                |row| row.get(0),
            )
            .optional()
        })
        .await
    }

    pub async fn finish_reused(&self, job: JobId, run: RunId) -> Result<(), Error> {
        self.call(move |conn| {
            conn.execute(
                "UPDATE jobs SET state = 'succeeded', result_run_id = ?1, updated_at = ?2
                 WHERE id = ?3",
                params![run, now_ms(), job],
            )
            .map(drop)
        })
        .await
    }

    pub async fn stored_result(&self, job: JobId) -> Result<StoredJob, Error> {
        self.call(move |conn| {
            let (state, reason, run_id, known_head) = conn.query_row(
                "SELECT state, failure_reason, coalesce(result_run_id,
                 (SELECT max(id) FROM runs WHERE job_id = jobs.id)),
                 (SELECT head_sha FROM targets WHERE id = jobs.target_id)
                 FROM jobs WHERE id = ?1",
                [job],
                |row| {
                    let run: Option<RunId> = row.get(2)?;
                    Ok((row.get(0)?, row.get(1)?, run, row.get(3)?))
                },
            )?;
            let reused = match run_id {
                Some(run) => conn.query_row(
                    "SELECT job_id != ?1 FROM runs WHERE id = ?2",
                    params![job, run],
                    |r| r.get(0),
                )?,
                None => false,
            };
            let run = run_id
                .map(|run| read::run(conn, run))
                .transpose()?
                .flatten();
            Ok(StoredJob {
                state,
                reason,
                run,
                reused,
                known_head,
            })
        })
        .await
    }
}
