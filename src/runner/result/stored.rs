use super::{JobEnd, ReviewResult};
use crate::store::result::{StoredJob, StoredRun};

impl ReviewResult {
    pub fn from_stored(stored: StoredJob, reused: bool) -> Self {
        match (stored.state.as_str(), stored.run) {
            (
                "succeeded",
                Some(StoredRun::Succeeded {
                    run_id,
                    head_sha,
                    findings,
                    limitation,
                    future_improvements,
                    warnings,
                }),
            ) => Self::Succeeded {
                run_id,
                head_sha,
                findings,
                limitation,
                future_improvements,
                warnings,
                reused: reused || stored.reused,
            },
            (
                _,
                Some(StoredRun::Failed {
                    run_id,
                    kind,
                    message,
                }),
            ) => Self::RunFailed {
                run_id,
                kind,
                message,
                job_end: JobEnd::Failed,
            },
            _ => Self::JobFailed {
                reason: stored
                    .reason
                    .unwrap_or_else(|| format!("job is {}", stored.state)),
            },
        }
    }
}
