use super::Store;
use crate::error::Error;

pub(crate) struct TargetStatus {
    pub label: String,
    pub tracking: String,
    pub latest: String,
}

struct Row {
    job_state: Option<String>,
    failure_reason: Option<String>,
    run_state: Option<String>,
    result: Option<String>,
    failure_kind: Option<String>,
    cause: Option<String>,
    findings: i64,
}

fn latest(row: &Row) -> String {
    let none = || "none".to_string();
    match row.run_state.as_deref() {
        Some("succeeded") if row.result.as_deref() == Some("findings") => {
            format!("succeeded (findings: {})", row.findings)
        }
        Some("succeeded") => format!("succeeded ({})", row.result.clone().unwrap_or_else(none)),
        Some("failed") => match (row.failure_kind.as_deref(), row.cause.as_deref()) {
            (Some("interrupted"), Some(cause)) => format!("failed (interrupted: {cause})"),
            (Some(kind), _) => format!("failed ({kind})"),
            (None, _) => "failed".to_string(),
        },
        Some(state) => state.to_string(),
        None => match row.job_state.as_deref() {
            Some("failed") => {
                let reason = row.failure_reason.clone().unwrap_or_else(none);
                format!("failed before run: {reason}")
            }
            Some(state) => state.to_string(),
            None => "queued".to_string(),
        },
    }
}

const SQL: &str = "SELECT t.owner, t.repo, t.number, t.tracking,
       j.state, j.failure_reason, r.state, r.result, r.failure_kind, r.interrupted_cause,
       (SELECT count(*) FROM findings f WHERE f.run_id = r.id)
    FROM targets t
    LEFT JOIN jobs j ON j.id = (SELECT max(id) FROM jobs WHERE target_id = t.id)
    LEFT JOIN runs r ON r.id = (SELECT max(id) FROM runs WHERE job_id = j.id)
    ORDER BY t.id";

impl Store {
    pub async fn target_statuses(&self) -> Result<Vec<TargetStatus>, Error> {
        self.call(|conn| {
            let mut statement = conn.prepare(SQL)?;
            let rows = statement.query_map([], |r| {
                let label = format!(
                    "{}/{}#{}",
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?
                );
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
                    label,
                    tracking: r.get(3)?,
                    latest: latest(&row),
                })
            })?;
            rows.collect()
        })
        .await
    }
}
