use super::steps::{Prepared, prepare};
use super::{Deps, record_agent_process};
use crate::agent::{Agent, AgentOutcome, FailureDetail, FailureKind};
use crate::cancel::{self, CancelRx};
use crate::gh::{Gh, PrMeta};
use crate::git::Git;
use crate::paths::Paths;
use crate::pr_url::PrRef;
use crate::store::records::{Metrics, Revision};
use crate::store::{RunId, Store, TargetId};
use tokio::sync::mpsc;

pub(super) async fn run_agent<G: Git, H: Gh, A: Agent>(
    paths: &Paths,
    store: &Store,
    deps: &Deps<G, H, A>,
    ids: (&PrRef, TargetId, RunId),
    revisions: (&PrMeta, &Revision),
    cancel: &CancelRx,
) -> (AgentOutcome, Prepared) {
    let run = ids.2;
    let mut stop = cancel.clone();
    let preparing = prepare(paths, store, deps, ids, revisions);
    let (request, mut prepared) = match cancel::raced(&mut stop, preparing).await {
        None => {
            let cancelled = AgentOutcome::Cancelled {
                metrics: Metrics::default(),
            };
            return (cancelled, Prepared::default());
        }
        Some(Ok(ready)) => ready,
        Some(Err(error)) => {
            let detail = FailureDetail {
                message: format!("prepare: {error}"),
                ..FailureDetail::default()
            };
            let kind = FailureKind::CliError;
            let metrics = Metrics::default();
            return (
                AgentOutcome::Failed {
                    kind,
                    detail,
                    metrics,
                },
                Prepared::default(),
            );
        }
    };
    let (hook, pids) = mpsc::unbounded_channel();
    let request = request.with_cancel(cancel.clone()).with_spawn_hook(hook);
    let running = async {
        let outcome = deps.agent.run(&request).await;
        drop(request);
        outcome
    };
    let (outcome, problems) = tokio::join!(running, record_agent_process(store, run, pids));
    prepared.warnings.extend(problems);
    (outcome, prepared)
}
