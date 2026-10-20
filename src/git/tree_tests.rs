use super::Git;
use super::tests::fetched;
use crate::error::Error;

const HEAD: &str = "refs/diffmate/pull/1/head";

#[tokio::test]
async fn tree_entry_reports_mode_oid_and_size_or_nothing() {
    let (clone, up) = fetched().await;

    let file = clone.cli.tree_entry(&clone.repo, &up.head, "a.txt").await;
    let link = clone.cli.tree_entry(&clone.repo, &up.head, "link").await;
    let absent = clone
        .cli
        .tree_entry(&clone.repo, &up.head, "REVIEW.md")
        .await;

    let file = file.unwrap().unwrap();
    assert_eq!((file.mode.as_str(), file.size), ("100644", 4));
    assert_eq!(file.oid.len(), 40);
    let link = link.unwrap().unwrap();
    assert_eq!((link.mode.as_str(), link.size), ("120000", 13));
    assert!(absent.unwrap().is_none());
    let base = clone.cli.tree_entry(&clone.repo, &up.base, "link").await;
    assert!(base.unwrap().is_none());
}

#[tokio::test]
async fn tree_entry_fails_for_an_unknown_commit() {
    let (clone, _) = fetched().await;
    let unknown = "0".repeat(40);

    let result = clone.cli.tree_entry(&clone.repo, &unknown, "a.txt").await;

    assert!(matches!(result, Err(Error::Command { .. })));
}

#[tokio::test]
async fn blob_returns_the_content_up_to_the_cap() {
    let (clone, up) = fetched().await;
    let file = clone.cli.tree_entry(&clone.repo, HEAD, "a.txt").await;
    let oid = file.unwrap().unwrap().oid;
    let link = clone.cli.tree_entry(&clone.repo, &up.head, "link").await;
    let link_oid = link.unwrap().unwrap().oid;

    let whole = clone.cli.blob(&clone.repo, &oid, 100).await.unwrap();
    let cut = clone.cli.blob(&clone.repo, &oid, 2).await.unwrap();
    let target = clone.cli.blob(&clone.repo, &link_oid, 100).await.unwrap();

    assert_eq!(whole, b"two\n");
    assert_eq!(cut, b"tw");
    assert_eq!(target, b"../../outside");
    let missing = clone.cli.blob(&clone.repo, &"1".repeat(40), 10).await;
    assert!(matches!(missing, Err(Error::Command { .. })));
}
