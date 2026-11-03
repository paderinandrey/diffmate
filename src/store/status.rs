use super::Store;
use crate::error::Error;
use crate::pr_url::PrRef;

mod latest;
use latest::{Row, latest};

pub(crate) struct TargetStatus {
    pub pr: PrRef,
    pub label: String,
    pub tracking: String,
    pub latest: String,
    pub draft_head: Option<String>,
    pub known_head: Option<(String, i64)>,
    #[expect(dead_code)]
    pub retry: RetryView,
}

#[expect(dead_code)]
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
       EXISTS (SELECT 1 FROM runs WHERE job_id = j.id AND state = 'failed')
    FROM targets t
    LEFT JOIN jobs j ON j.id = (SELECT max(id) FROM jobs WHERE target_id = t.id)
    LEFT JOIN runs r ON r.id = (SELECT max(id) FROM runs WHERE job_id = j.id)
    ORDER BY t.id";

impl Store {
    pub async fn target_statuses(&self) -> Result<Vec<TargetStatus>, Error> {
        self.call(|conn| {
            let mut statement = conn.prepare(SQL)?;
            let rows = statement.query_map([], |r| {
                let number: i64 = r.get(2)?;
                let pr = PrRef {
                    owner: r.get(0)?,
                    repo: r.get(1)?,
                    number: u64::try_from(number)
                        .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(2, number))?,
                };
                let row = Row {
                    job_state: r.get(4)?,
                    failure_reason: r.get(5)?,
                    run_state: r.get(6)?,
                    result: r.get(7)?,
                    failure_kind: r.get(8)?,
                    cause: r.get(9)?,
                    findings: r.get(10)?,
                };
                Ok(TargetStatus {
                    label: pr.label(),
                    pr,
                    tracking: r.get(3)?,
                    latest: latest(&row),
                    draft_head: r.get(13)?,
                    known_head: r.get::<_, Option<String>>(11)?.zip(r.get(12)?),
                    retry: RetryView {
                        job_state: row.job_state,
                        origin: r.get(14)?,
                        retry_count: r.get(15)?,
                        max_retries: r.get(16)?,
                        next_attempt_at: r.get(17)?,
                        has_failed_run: r.get(18)?,
                    },
                })
            })?;
            rows.collect()
        })
        .await
    }
}
