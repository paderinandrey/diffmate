use super::super::Absent;
use super::fixture::{Fix, absent_open, closed, pr, seen};

#[tokio::test]
async fn a_new_request_tracks_and_a_repeat_does_not_change_tracking() {
    let fix = Fix::open().await;
    fix.seed(2, true, true).await;
    fix.seed(3, false, true).await;
    fix.poll_seen().await;
    let fresh = fix.target(1).await.unwrap();
    assert_eq!(fresh.tracking, "tracked");
    assert!(fresh.seen);
    assert_eq!(
        (fresh.head.as_deref(), fresh.checked),
        (Some("aaa"), Some(1_000))
    );
    let kept = fix.target(2).await.unwrap();
    assert_eq!((kept.tracking.as_str(), kept.seen), ("tracked", true));
    let discarded = fix.target(3).await.unwrap();
    assert_eq!(
        (discarded.tracking.as_str(), discarded.seen),
        ("untracked", true)
    );
    let draft = fix.target(4).await.unwrap();
    assert_eq!((draft.tracking.as_str(), draft.seen), ("tracked", true));
    let known = fix.store.known_targets().await.unwrap();
    let row = known.iter().find(|row| row.pr.number == 1).unwrap();
    assert_eq!(known.len(), 4);
    assert_eq!(row.tracking, "tracked");
    assert!(row.request_seen);
    assert_eq!(row.queued_head.as_deref(), Some("aaa"));
}

impl Fix {
    async fn poll_seen(&self) {
        self.poll(
            vec![
                seen(1, false, Some("aaa")),
                seen(2, false, None),
                seen(3, true, Some("bbb")),
                seen(4, true, Some("ccc")),
            ],
            vec![],
            1_000,
            60_000,
        )
        .await;
    }
}

#[tokio::test]
async fn absence_writes_only_the_column_for_that_variant() {
    let fix = Fix::open().await;
    fix.seed(1, true, true).await;
    fix.seed(2, true, true).await;
    fix.seed(3, true, false).await;
    fix.seed(4, false, true).await;
    fix.seed(5, true, true).await;
    fix.seed(6, false, false).await;
    fix.poll(
        vec![],
        vec![
            (pr(1), absent_open(false, Some("new"))),
            (pr(2), closed(None)),
            (pr(3), closed(None)),
            (pr(4), Absent::Skip),
            (pr(8), closed(None)),
        ],
        1_000,
        60_000,
    )
    .await;
    expect(&fix, 1, "tracked", false).await;
    let open = fix.target(1).await.unwrap();
    assert_eq!(
        (open.head.as_deref(), open.checked),
        (Some("new"), Some(1_000))
    );
    expect(&fix, 2, "untracked", false).await;
    expect(&fix, 3, "untracked", false).await;
    expect(&fix, 4, "untracked", true).await;
    expect(&fix, 5, "tracked", true).await;
    expect(&fix, 6, "untracked", false).await;
    assert!(fix.target(8).await.is_none());
}

async fn expect(fix: &Fix, number: u64, tracking: &str, seen: bool) {
    let row = fix.target(number).await.unwrap();
    assert_eq!((row.tracking.as_str(), row.seen), (tracking, seen));
}
