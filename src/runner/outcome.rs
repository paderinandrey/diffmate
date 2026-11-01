use super::{ReviewResult, job_failed};
use crate::agent::{AgentOutcome, FailureDetail, FailureKind, ReviewOutput};
use crate::cancel::StopReason;
use crate::normalize::Normalized;
use crate::store::jobs::EnqueuedJob;
use crate::store::records::{DraftRecord, FindingRecord, Revision};
use crate::store::{RunId, Store};
use failed::failed;

mod failed;
#[cfg(test)]
mod tests;

fn draft_record(
    output: ReviewOutput,
    raw: serde_json::Value,
    findings: Vec<FindingRecord>,
) -> DraftRecord {
    DraftRecord {
        original_output: raw,
        limitation: output.limitation,
        future_improvements: output.future_improvements,
        findings,
    }
}

pub(super) async fn store(
    store: &Store,
    (job, run, revision): (&EnqueuedJob, RunId, &Revision),
    (outcome, normalized): (AgentOutcome, Normalized),
    cleanup_warnings: Vec<String>,
) -> ReviewResult {
    match outcome {
        AgentOutcome::Succeeded {
            output,
            raw,
            metrics,
        } => {
            let mut warnings = normalized.warnings;
            warnings.extend(cleanup_warnings);
            let draft = draft_record(output, raw, normalized.findings);
            let result = ReviewResult::Succeeded {
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
        AgentOutcome::Cancelled { metrics } => {
            let stop = store.stop_reason(job.job_id).await.unwrap_or(None);
            let (kind, message) = match stop {
                Some(StopReason::Timeout) => {
                    (FailureKind::Timeout, "stopped by the run timeout".into())
                }
                Some(cause) => (
                    FailureKind::Interrupted,
                    format!("stopped: interrupted ({})", cause.as_str()),
                ),
                None => (FailureKind::Interrupted, "interrupted by signal".into()),
            };
            let detail = FailureDetail {
                message,
                ..FailureDetail::default()
            };
            failed(
                store,
                (job, run),
                (kind, detail, metrics),
                stop,
                cleanup_warnings,
            )
            .await
        }
        AgentOutcome::Failed {
            kind,
            detail,
            metrics,
        } => {
            failed(
                store,
                (job, run),
                (kind, detail, metrics),
                None,
                cleanup_warnings,
            )
            .await
        }
    }
}
