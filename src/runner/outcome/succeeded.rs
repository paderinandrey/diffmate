use super::{ReviewResult, job_failed};
use crate::agent::ReviewOutput;
use crate::crash::crash_point;
use crate::normalize::Normalized;
use crate::store::jobs::EnqueuedJob;
use crate::store::records::{DraftRecord, Metrics};
use crate::store::{RunId, Store};

pub(super) async fn store(
    store: &Store,
    (job, run): (&EnqueuedJob, RunId),
    (output, raw, metrics): (ReviewOutput, serde_json::Value, Metrics),
    normalized: Normalized,
    cleanup_warnings: Vec<String>,
) -> ReviewResult {
    let mut warnings = normalized.warnings;
    warnings.extend(cleanup_warnings);
    let draft = DraftRecord {
        original_output: raw,
        limitation: output.limitation,
        future_improvements: output.future_improvements,
        findings: normalized.findings,
    };
    match store
        .finish_succeeded(run, job.job_id, draft, warnings, metrics)
        .await
    {
        Ok(()) => {
            crash_point!("stored");
            match store.stored_result(job.job_id).await {
                Ok(stored) => ReviewResult::from_stored(stored, false),
                Err(error) => job_failed(store, job, error.to_string()).await,
            }
        }
        Err(error) => job_failed(store, job, format!("storing run {run}: {error}")).await,
    }
}
