use super::Git;
use super::fixture::git;
use super::tests::fetched;
use std::fs;

#[tokio::test]
async fn fetch_leaves_exactly_the_two_namespaced_refs() {
    let (clone, up) = fetched().await;
    let refs = git(
        &clone.repo,
        &["for-each-ref", "--format=%(refname) %(objectname)"],
    );
    let expected = format!(
        "refs/diffmate/base/main {}\nrefs/diffmate/pull/1/head {}",
        up.base, up.head
    );
    assert_eq!(refs, expected);
}

#[tokio::test]
async fn a_pr_symlink_is_checked_out_as_a_plain_file() {
    let (clone, up) = fetched().await;
    let worktree = clone.tmp.path().join("wt");
    clone
        .cli
        .worktree_add(&clone.repo, &worktree, &up.head)
        .await
        .unwrap();
    let meta = fs::symlink_metadata(worktree.join("link")).unwrap();
    assert!(meta.file_type().is_file());
    assert_eq!(
        fs::read_to_string(worktree.join("link")).unwrap(),
        "../../outside"
    );
}
