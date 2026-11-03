use super::records::JobOrigin;
use super::{JobId, Store, TargetId, now_ms};
use crate::cancel::StopReason;
use crate::error::Error;
use rusqlite::params;

mod attach;

pub(crate) struct EnqueuedJob {
    pub target_id: TargetId,
    pub job_id: JobId,
}

pub(crate) struct EnqueueRequest {
    pub origin: JobOrigin,
    pub force: bool,
}

pub(crate) struct Enqueued {
    pub job: EnqueuedJob,
    #[expect(dead_code)]
    pub attached: bool,
}

pub(crate) struct StartedJob {
    pub force: bool,
}

impl Store {
    pub async fn set_stop_reason(&self, job: JobId, reason: StopReason) -> Result<bool, Error> {
        self.call(move |conn| {
            let sql = "UPDATE jobs SET stop_reason = ?1, updated_at = ?2
                       WHERE id = ?3 AND stop_reason IS NULL";
            conn.execute(sql, params![reason.as_str(), now_ms(), job])
                .map(|changed| changed == 1)
        })
        .await
    }

    pub async fn stop_reason(&self, job: JobId) -> Result<Option<StopReason>, Error> {
        self.call(move |conn| {
            let sql = "SELECT stop_reason FROM jobs WHERE id = ?1";
            let reason: Option<String> = conn.query_row(sql, [job], |row| row.get(0))?;
            Ok(reason.as_deref().and_then(StopReason::parse))
        })
        .await
    }

    pub async fn start_job(&self, job: JobId) -> Result<StartedJob, Error> {
        self.call(move |conn| {
            let sql =
                "UPDATE jobs SET state = 'running', updated_at = ?1 WHERE id = ?2 RETURNING force";
            conn.query_row(sql, params![now_ms(), job], |row| {
                Ok(StartedJob { force: row.get(0)? })
            })
        })
        .await
    }

    pub async fn fail_job(&self, job: JobId, reason: String) -> Result<(), Error> {
        self.call(move |conn| {
            let sql = "UPDATE jobs SET state = 'failed', failure_reason = ?1, updated_at = ?2
                       WHERE id = ?3";
            conn.execute(sql, params![reason, now_ms(), job]).map(drop)
        })
        .await
    }
}
