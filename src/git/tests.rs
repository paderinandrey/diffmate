use super::fixture::{Upstream, cli, git, upstream};
use super::{Git, GitCli};
use crate::error::Error;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

pub(super) struct Clone {
    pub tmp: TempDir,
    pub cli: GitCli,
    pub repo: PathBuf,
    pub hooks: PathBuf,
}

pub(super) fn clone() -> Clone {
    let tmp = TempDir::new().unwrap();
    let (repo, hooks) = (tmp.path().join("r.git"), tmp.path().join("hooks"));
    Clone {
        cli: cli(tmp.path()),
        tmp,
        repo,
        hooks,
    }
}

pub(super) async fn fetched() -> (Clone, Upstream) {
    let clone = clone();
    let up = upstream(clone.tmp.path());
    let url = up.path.to_str().unwrap();
    clone
        .cli
        .ensure_clone(&clone.repo, url, &clone.hooks)
        .await
        .unwrap();
    clone.cli.fetch_pr(&clone.repo, 1, "main").await.unwrap();
    (clone, up)
}

#[tokio::test]
async fn a_non_empty_hooks_directory_fails_the_clone() {
    let clone = clone();
    fs::create_dir(&clone.hooks).unwrap();
    fs::write(clone.hooks.join("pre-commit"), "#!/bin/sh\n").unwrap();
    let result = clone
        .cli
        .ensure_clone(&clone.repo, "file:///x", &clone.hooks)
        .await;
    assert!(matches!(result, Err(Error::HooksNotEmpty(_))));
}

#[tokio::test]
async fn an_unsafe_base_branch_fails_before_any_ref_is_fetched() {
    let clone = clone();
    let up = upstream(clone.tmp.path());
    let url = up.path.to_str().unwrap();
    clone
        .cli
        .ensure_clone(&clone.repo, url, &clone.hooks)
        .await
        .unwrap();
    let result = clone
        .cli
        .fetch_pr(&clone.repo, 1, "main:refs/heads/evil")
        .await;
    assert!(matches!(result, Err(Error::UnsafeBranch(_))));
    assert_eq!(git(&clone.repo, &["for-each-ref"]), "");
}

#[tokio::test]
async fn a_base_branch_replaced_by_a_conflicting_name_still_fetches() {
    let (clone, up) = fetched().await;
    let list = [
        "for-each-ref",
        "--format=%(refname:lstrip=3)",
        "refs/diffmate/base/",
    ];
    let rounds = [
        (vec![], vec!["release/v1", "release/v2"]),
        (vec!["release/v1", "release/v2"], vec!["release"]),
        (vec!["release"], vec!["release/v3"]),
    ];
    for (gone, created) in rounds {
        for name in gone {
            let refname = format!("refs/heads/{name}");
            git(&up.path, &["update-ref", "-d", &refname]);
        }
        for name in &created {
            let refname = format!("refs/heads/{name}");
            git(&up.path, &["update-ref", &refname, &up.base]);
            clone.cli.fetch_pr(&clone.repo, 1, name).await.unwrap();
        }
        let expected = ["main"].iter().chain(&created).copied();
        assert_eq!(
            git(&clone.repo, &list),
            expected.collect::<Vec<_>>().join("\n")
        );
    }
}
