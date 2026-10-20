use super::Deps;
use crate::agent::{Agent, AgentOutcome, AgentRequest, FailureDetail, FailureKind};
use crate::context::{self, ContextInputs};
use crate::diff::{self, DiffIndex};
use crate::error::Error;
use crate::gh::{Gh, PrMeta};
use crate::git::Git;
use crate::paths::{Paths, create_private_dir_async};
use crate::pr_url::PrRef;
use crate::store::records::{Metrics, Revision};
use crate::store::{RunId, Store, TargetId};
use std::collections::HashSet;

#[derive(Default)]
pub(super) struct Prepared {
    pub index: DiffIndex,
    pub previous_ids: HashSet<String>,
    pub warnings: Vec<String>,
}

pub(super) async fn prepare<G: Git, H: Gh, A>(
    paths: &Paths,
    store: &Store,
    deps: &Deps<G, H, A>,
    (pr, target, run): (&PrRef, TargetId, RunId),
    (meta, revision): (&PrMeta, &Revision),
) -> Result<(AgentRequest, Prepared), Error> {
    let repo = paths.repo_dir(&pr.owner, &pr.repo);
    let (worktree, context_dir) = (paths.worktree(run), paths.context_dir(run));
    let made = create_private_dir_async(paths.run_dir(run)).await;
    made.map_err(Error::io("create run directory"))?;
    deps.git
        .worktree_add(&repo, &worktree, &revision.head_sha)
        .await?;
    let diff = deps
        .git
        .diff(&repo, &revision.merge_base_sha, &revision.head_sha)
        .await?;
    let inputs = ContextInputs {
        dir: &context_dir,
        pr,
        meta,
        revision,
        diff,
        repo: &repo,
        git: &deps.git,
        gh: &deps.gh,
        target,
        run,
        store,
    };
    let written = context::write(inputs).await?;
    let prompt_file = paths.prompt_file(run);
    context::write_prompt(&prompt_file).await?;
    let prepared = Prepared {
        index: diff::parse(&written.diff_as_written),
        previous_ids: written.previous_ids,
        warnings: written.warnings,
    };
    Ok((
        AgentRequest::new(worktree, context_dir, prompt_file),
        prepared,
    ))
}

pub(super) async fn run_agent<G: Git, H: Gh, A: Agent>(
    paths: &Paths,
    store: &Store,
    deps: &Deps<G, H, A>,
    ids: (&PrRef, TargetId, RunId),
    revisions: (&PrMeta, &Revision),
) -> (AgentOutcome, Prepared) {
    match prepare(paths, store, deps, ids, revisions).await {
        Ok((request, prepared)) => (deps.agent.run(&request).await, prepared),
        Err(error) => {
            let detail = FailureDetail {
                message: format!("prepare: {error}"),
                ..FailureDetail::default()
            };
            let failed = AgentOutcome::Failed {
                kind: FailureKind::CliError,
                detail,
                metrics: Metrics::default(),
            };
            (failed, Prepared::default())
        }
    }
}
