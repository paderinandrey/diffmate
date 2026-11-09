use super::fixture::{Fix, absent_open, pr};

const DEBOUNCE: i64 = 60_000;

#[tokio::test]
async fn absent_open_arms_only_a_target_that_stays_tracked() {
    let fix = Fix::open().await;
    fix.seed(1, true, true).await;
    fix.seed(2, false, true).await;
    fix.seed(3, true, true).await;
    fix.set_heads(1, None, Some("old"), Some(5)).await;
    fix.poll(
        vec![],
        vec![
            (pr(1), absent_open(false, Some("bbb"))),
            (pr(2), absent_open(false, Some("bbb"))),
            (pr(3), absent_open(true, Some("ccc"))),
        ],
        1_000,
        DEBOUNCE,
    )
    .await;
    let jobs = fix.jobs(1).await;
    assert_eq!(jobs.len(), 1);
    assert_eq!((jobs[0].retry, jobs[0].next), (0, Some(61_000)));
    let tracked = fix.target(1).await.unwrap();
    assert_eq!(tracked.tracking, "tracked");
    assert!(!tracked.seen);
    assert_eq!(tracked.queued_head.as_deref(), Some("bbb"));
    assert_eq!(
        (tracked.head.as_deref(), tracked.checked),
        (Some("bbb"), Some(1_000))
    );
    assert!(fix.jobs(2).await.is_empty());
    assert!(fix.jobs(3).await.is_empty());
    let hidden = fix.target(2).await.unwrap();
    assert_eq!(
        (hidden.tracking.as_str(), hidden.seen),
        ("untracked", false)
    );
}
