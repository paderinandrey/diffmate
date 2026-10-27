use super::Deps;
use crate::error::Error;
use crate::gh::{Gh, PrMeta};
use crate::git::Git;
use crate::paths::Paths;
use crate::pr_url::{self, PrRef};
use crate::store::records::Revision;

pub(super) async fn resolve<G: Git, H: Gh, A>(
    paths: &Paths,
    deps: &Deps<G, H, A>,
    pr: &PrRef,
) -> Result<(PrMeta, Revision), Error> {
    let meta = deps.gh.pr_view(pr).await?;
    let canonical = pr_url::parse(&meta.url)?;
    if canonical != *pr {
        return Err(Error::Canonical(canonical.label()));
    }
    let repo = paths.repo_dir(&pr.owner, &pr.repo);
    deps.git
        .ensure_clone(&repo, &pr.remote_url(), &paths.empty_hooks_dir())
        .await?;
    deps.git.fetch_pr(&repo, pr.number, &meta.base_ref).await?;
    let head = deps
        .git
        .rev_parse(&repo, &format!("refs/diffmate/pull/{}/head", pr.number))
        .await?;
    let base_ref = &meta.base_ref;
    let tip = deps
        .git
        .rev_parse(&repo, &format!("refs/diffmate/base/{base_ref}"))
        .await?;
    let merge_base_sha = deps.git.merge_base(&repo, &tip, &head).await?;
    let revision = Revision {
        head_sha: head,
        merge_base_sha,
        base_tip_sha: tip,
        base_ref: base_ref.clone(),
    };
    Ok((meta, revision))
}
