use crate::cancel::StopReason;
use crate::recovery::rules::{Failure, JobNext, RetryState, retry_delay};
use crate::store::records::JobOrigin;
use crate::store::{JobId, RunId};
use rusqlite::{Transaction, params};

pub(super) struct RunningJob {
    pub id: JobId,
    pub origin: JobOrigin,
    pub stop: Option<StopReason>,
    pub retry: RetryState,
}

pub(super) fn jobs(tx: &Transaction) -> rusqlite::Result<Vec<RunningJob>> {
    let mut query = tx.prepare(
        "SELECT id, origin, stop_reason, retry_count, max_retries
        FROM jobs WHERE state = 'running' AND kind = 'review' ORDER BY id",
    )?;
    query
        .query_map([], |row| {
            let stop: Option<String> = row.get(2)?;
            Ok(RunningJob {
                id: row.get(0)?,
                origin: JobOrigin::parse(&row.get::<_, String>(1)?)?,
                stop: stop.as_deref().and_then(StopReason::parse),
                retry: RetryState {
                    retry_count: row.get(3)?,
                    max_retries: row.get(4)?,
                },
            })
        })?
        .collect()
}

pub(super) fn runs(tx: &Transaction, now: i64) -> rusqlite::Result<Vec<(RunId, String)>> {
    let mut query = tx.prepare(
        "SELECT r.id, j.stop_reason FROM runs r LEFT JOIN jobs j
        ON j.id = r.job_id AND j.state = 'running' AND j.kind = 'review'
        WHERE r.state = 'running' ORDER BY r.id",
    )?;
    let rows = query.query_map([], |row| {
        Ok((row.get::<_, RunId>(0)?, row.get::<_, Option<String>>(1)?))
    })?;
    let rows: Vec<_> = rows.collect::<rusqlite::Result<_>>()?;
    let mut descriptions = Vec::new();
    for (run, stop) in rows {
        let failure = Failure::from_stop(stop.as_deref().and_then(StopReason::parse));
        let (kind, cause) = match failure {
            Failure::Timeout => ("timeout", None),
            Failure::Shutdown => ("interrupted", Some("shutdown")),
            Failure::UserCancel => ("interrupted", Some("user_cancel")),
            _ => ("interrupted", Some("crash")),
        };
        tx.execute(
            "UPDATE runs SET state = 'failed', failure_kind = ?1,
            interrupted_cause = ?2, finished_at = ?3 WHERE id = ?4",
            params![kind, cause, now, run],
        )?;
        descriptions.push((run, failure.text().into()));
    }
    Ok(descriptions)
}

pub(super) fn description(next: &JobNext, failure: &Failure) -> String {
    match next {
        JobNext::Fail { reason, .. } => format!("failed ({reason})"),
        JobNext::Requeue {
            retry_count,
            max_retries: Some(max),
            next_attempt_at: Some(_),
        } => {
            let delay = retry_delay(*retry_count).as_secs();
            format!("requeued: retry {retry_count} of {max} after {delay}s")
        }
        JobNext::Requeue { .. } => format!("requeued ({})", failure.text()),
    }
}
