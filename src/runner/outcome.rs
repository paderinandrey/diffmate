use super::{ReviewResult, job_failed};
use crate::agent::{AgentOutcome, FailureDetail, FailureKind, ReviewOutput};
use crate::normalize::Normalized;
use crate::store::jobs::EnqueuedJob;
use crate::store::records::{DraftRecord, FindingRecord, Metrics, Revision};
use crate::store::{RunId, Store};

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
            let detail = FailureDetail {
                message: "interrupted by signal".into(),
                ..FailureDetail::default()
            };
            failed(
                store,
                (job, run),
                (FailureKind::Interrupted, detail, metrics),
                cleanup_warnings,
            )
            .await
        }
        AgentOutcome::Failed {
            kind,
            detail,
            metrics,
        } => failed(store, (job, run), (kind, detail, metrics), cleanup_warnings).await,
    }
}

async fn failed(
    store: &Store,
    (job, run): (&EnqueuedJob, RunId),
    (kind, detail, metrics): (FailureKind, FailureDetail, Metrics),
    cleanup_warnings: Vec<String>,
) -> ReviewResult {
    let mut message = detail.message.clone();
    for warning in cleanup_warnings {
        message = format!("{message}; {warning}");
    }
    match store
        .finish_failed(run, job.job_id, kind, detail, metrics)
        .await
    {
        Ok(()) if matches!(kind, FailureKind::Interrupted) => {
            ReviewResult::Cancelled { run_id: Some(run) }
        }
        Ok(()) => ReviewResult::RunFailed {
            run_id: run,
            kind,
            message,
        },
        Err(error) => job_failed(store, job, format!("storing run {run}: {error}")).await,
    }
}
