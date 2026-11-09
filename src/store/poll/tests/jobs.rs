use super::super::{Absent, SeenRequest};
use super::fixture::{self, Fix, pr};
use crate::pr_url::PrRef;
use crate::store::jobs::EnqueueRequest;
use crate::store::records::JobOrigin;
use rusqlite::params;

pub(super) struct Job {
    pub id: i64,
    pub state: String,
    pub origin: String,
    pub force: bool,
    pub retry: i64,
    pub next: Option<i64>,
}

impl Fix {
    pub(super) async fn jobs(&self, number: u64) -> Vec<Job> {
        self.store
            .call(move |conn| {
                let mut statement = conn.prepare(
                    "SELECT j.id, j.state, j.origin, j.force, j.retry_count, j.next_attempt_at
                     FROM jobs j JOIN targets t ON t.id = j.target_id
                     WHERE t.number = ?1 ORDER BY j.id",
                )?;
                let mapped = statement.query_map([i64::try_from(number).unwrap()], |row| {
                    Ok(Job {
                        id: row.get(0)?,
                        state: row.get(1)?,
                        origin: row.get(2)?,
                        force: row.get(3)?,
                        retry: row.get(4)?,
                        next: row.get(5)?,
                    })
                })?;
                mapped.collect()
            })
            .await
            .unwrap()
    }

    pub(super) async fn enqueue(&self, number: u64, automatic: bool) -> i64 {
        let origin = if automatic {
            JobOrigin::Automatic
        } else {
            JobOrigin::Manual
        };
        self.store
            .enqueue_review(
                &pr(number),
                &fixture::process(),
                EnqueueRequest {
                    origin,
                    force: false,
                },
            )
            .await
            .unwrap()
            .job
            .job_id
    }

    pub(super) async fn set_job(&self, job: i64, state: &str, retry: i64, next: Option<i64>) {
        let state = state.to_string();
        self.store
            .call(move |conn| {
                conn.execute(
                    "UPDATE jobs SET state = ?1, retry_count = ?2, next_attempt_at = ?3 WHERE id = ?4",
                    params![state, retry, next, job],
                )
                .map(drop)
            })
            .await
            .unwrap();
    }

    pub(super) async fn poll(
        &self,
        seen: Vec<SeenRequest>,
        absent: Vec<(PrRef, Absent)>,
        now: i64,
        debounce: i64,
    ) {
        self.store
            .apply_poll(seen, absent, now, debounce, &fixture::process())
            .await
            .unwrap();
    }
}
