use super::{Store, TargetId};
use crate::error::Error;
use crate::pr_url::PrRef;

mod draft;
mod latest;
mod queue;
use latest::{Row, latest};

pub(crate) use draft::DraftView;
pub(crate) use queue::QueueRow;

pub(crate) struct TargetStatus {
    pub id: TargetId,
    pub pr: PrRef,
    pub label: String,
    pub tracking: String,
    pub latest: String,
    pub draft_head: Option<String>,
    pub known_head: Option<(String, i64)>,
    pub retry: RetryView,
    pub publication: String,
}

pub(crate) struct RetryView {
    pub job_state: Option<String>,
    pub origin: Option<String>,
    pub retry_count: i64,
    pub max_retries: Option<i64>,
    pub next_attempt_at: Option<i64>,
    pub has_failed_run: bool,
}

const SQL: &str = "SELECT t.owner, t.repo, t.number, t.tracking,
       j.state, j.failure_reason, r.state, r.result, r.failure_kind, r.interrupted_cause,
       (SELECT count(*) FROM findings f WHERE f.run_id = r.id),
       t.head_sha, t.head_checked_at,
       (SELECT head_sha FROM runs WHERE target_id = t.id AND state = 'succeeded'
        ORDER BY id DESC LIMIT 1),
       j.origin, coalesce(j.retry_count, 0), j.max_retries, j.next_attempt_at,
       EXISTS (SELECT 1 FROM runs WHERE job_id = j.id AND state = 'failed'),
       t.id,
       (SELECT p.state FROM publications p WHERE p.target_id = t.id
        ORDER BY p.created_at DESC LIMIT 1)
    FROM targets t
    LEFT JOIN jobs j ON j.id = (SELECT max(id) FROM jobs WHERE target_id = t.id AND kind = 'review')
    LEFT JOIN runs r ON r.id = coalesce(j.result_run_id,
        (SELECT max(id) FROM runs WHERE job_id = j.id))";

pub(super) fn read_status(row: &rusqlite::Row<'_>) -> rusqlite::Result<TargetStatus> {
    let number: i64 = row.get(2)?;
    let pr = PrRef {
        owner: row.get(0)?,
        repo: row.get(1)?,
        number: u64::try_from(number)
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(2, number))?,
    };
    let latest_row = Row {
        job_state: row.get(4)?,
        failure_reason: row.get(5)?,
        run_state: row.get(6)?,
        result: row.get(7)?,
        failure_kind: row.get(8)?,
        cause: row.get(9)?,
        findings: row.get(10)?,
    };
    let publication = row
        .get::<_, Option<String>>(20)?
        .unwrap_or_else(|| "none".to_string());
    Ok(TargetStatus {
        id: row.get(19)?,
        label: pr.label(),
        pr,
        tracking: row.get(3)?,
        latest: latest(&latest_row),
        draft_head: row.get(13)?,
        known_head: row.get::<_, Option<String>>(11)?.zip(row.get(12)?),
        retry: RetryView {
            job_state: latest_row.job_state,
            origin: row.get(14)?,
            retry_count: row.get(15)?,
            max_retries: row.get(16)?,
            next_attempt_at: row.get(17)?,
            has_failed_run: row.get(18)?,
        },
        publication,
    })
}

impl Store {
    pub async fn target_statuses(&self) -> Result<Vec<TargetStatus>, Error> {
        self.call(|conn| {
            let sql = format!("{SQL} ORDER BY t.id");
            let mut statement = conn.prepare(&sql)?;
            let rows = statement.query_map([], read_status)?;
            rows.collect()
        })
        .await
    }
}
