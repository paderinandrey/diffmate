use super::{ReviewResult, job_failed};
use crate::agent::{AgentOutcome, ReviewOutput};
use crate::store::jobs::EnqueuedJob;
use crate::store::records::{DraftRecord, FindingRecord, Revision};
use crate::store::{RunId, Store};

fn draft_record(
    output: ReviewOutput,
    raw: serde_json::Value,
    warnings: &mut Vec<String>,
) -> DraftRecord {
    let mut findings = Vec::new();
    for finding in output.findings {
        if finding.previous_finding_id.is_some() {
            warnings.push(format!(
                "previous_finding_id on \"{}\" ignored: no earlier run is known",
                finding.title
            ));
        }
        findings.push(FindingRecord {
            severity: finding.severity,
            path: finding.path,
            line: finding.line,
            side: finding.side,
            title: finding.title,
            body: finding.body,
            previous_finding_id: None,
        });
    }
    DraftRecord {
        original_output: raw,
        limitation: output.limitation,
        future_improvements: output.future_improvements,
        findings,
    }
}

pub(super) async fn store(
    store: &Store,
    job: &EnqueuedJob,
    run: RunId,
    revision: &Revision,
    outcome: AgentOutcome,
    mut warnings: Vec<String>,
) -> ReviewResult {
    match outcome {
        AgentOutcome::Succeeded {
            output,
            raw,
            metrics,
        } => {
            let draft = draft_record(output, raw, &mut warnings);
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
        AgentOutcome::Failed {
            kind,
            detail,
            metrics,
        } => {
            let mut message = detail.message.clone();
            for warning in warnings {
                message = format!("{message}; {warning}");
            }
            match store
                .finish_failed(run, job.job_id, kind, detail, metrics)
                .await
            {
                Ok(()) => ReviewResult::RunFailed {
                    run_id: run,
                    kind,
                    message,
                },
                Err(error) => job_failed(store, job, format!("storing run {run}: {error}")).await,
            }
        }
    }
}
