use super::{ReviewResult, job_failed};
use crate::agent::ReviewOutput;
use crate::normalize::Normalized;
use crate::store::jobs::EnqueuedJob;
use crate::store::records::{DraftRecord, Metrics, Revision};
use crate::store::{RunId, Store};

pub(super) async fn store(
    store: &Store,
    (job, run, revision): (&EnqueuedJob, RunId, &Revision),
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
    let result = ReviewResult::Succeeded {
        reused: false,
        run_id: run,
        head_sha: revision.head_sha.clone(),
        findings: draft.findings.clone(),
        limitation: draft.limitation.clone(),
        future_improvements: draft.future_improvements.clone(),
        warnings: warnings.clone(),
    };
    match store
        .finish_succeeded(run, job.job_id, draft, warnings, metrics)
        .await
    {
        Ok(()) => result,
        Err(error) => job_failed(store, job, format!("storing run {run}: {error}")).await,
    }
}
