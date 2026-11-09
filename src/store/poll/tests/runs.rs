use super::fixture::Fix;
use rusqlite::params;

impl Fix {
    pub(super) async fn add_run(&self, job: i64, head: &str, failed: bool) {
        let head = head.to_string();
        self.store
            .call(move |conn| {
                let sql = if failed {
                    "INSERT INTO runs (target_id, job_id, state, failure_kind, head_sha,
                     merge_base_sha, base_tip_sha, base_ref, created_at)
                     SELECT target_id, id, 'failed', 'invalid_output', ?2, 'm', 'b', 'main', 0
                     FROM jobs WHERE id = ?1"
                } else {
                    "INSERT INTO runs (target_id, job_id, state, result, head_sha,
                     merge_base_sha, base_tip_sha, base_ref, created_at)
                     SELECT target_id, id, 'succeeded', 'no_findings', ?2, 'm', 'b', 'main', 0
                     FROM jobs WHERE id = ?1"
                };
                conn.execute(sql, params![job, head]).map(drop)
            })
            .await
            .unwrap();
    }
}
