use super::fixture::{Fix, seen};

const DEBOUNCE: i64 = 60_000;

#[tokio::test]
async fn an_immediate_or_running_job_blocks_another() {
    let fix = Fix::open().await;
    fix.seed(1, true, true).await;
    fix.seed(2, true, true).await;
    let immediate = fix.enqueue(1, true).await;
    let running = fix.enqueue(2, true).await;
    fix.store.start_job(running).await.unwrap();
    fix.set_heads(2, Some("old"), Some("stored"), Some(9)).await;
    fix.poll(
        vec![seen(1, false, Some("bbb")), seen(2, false, Some("ccc"))],
        vec![],
        2_000,
        DEBOUNCE,
    )
    .await;
    let blocked = &fix.jobs(1).await[0];
    assert_eq!(fix.jobs(1).await.len(), 1);
    assert_eq!((blocked.id, blocked.next), (immediate, None));
    assert_eq!(fix.target(1).await.unwrap().queued_head, None);
    let live = &fix.jobs(2).await[0];
    assert_eq!(fix.jobs(2).await.len(), 1);
    assert_eq!(
        (live.id, live.state.as_str(), live.next),
        (running, "running", None)
    );
    let row = fix.target(2).await.unwrap();
    assert_eq!(row.queued_head.as_deref(), Some("old"));
    assert_eq!(
        (row.head.as_deref(), row.checked),
        (Some("ccc"), Some(2_000))
    );
}

#[tokio::test]
async fn draft_untracked_missing_and_reviewed_heads_do_not_queue() {
    let fix = Fix::open().await;
    fix.seed(1, true, true).await;
    fix.seed(2, false, true).await;
    fix.seed(3, true, true).await;
    fix.seed(4, true, true).await;
    let reviewed = fix.enqueue(4, true).await;
    fix.set_job(reviewed, "failed", 2, None).await;
    fix.add_run(reviewed, "aaa", true).await;
    fix.poll(
        vec![
            seen(1, true, Some("aaa")),
            seen(2, false, Some("aaa")),
            seen(3, false, None),
            seen(4, false, Some("aaa")),
        ],
        vec![],
        1_000,
        DEBOUNCE,
    )
    .await;
    assert!(fix.jobs(1).await.is_empty());
    assert!(fix.jobs(2).await.is_empty());
    assert!(fix.jobs(3).await.is_empty());
    assert_eq!(fix.target(2).await.unwrap().tracking, "untracked");
    assert_eq!(fix.jobs(4).await.len(), 1);
    assert_eq!(fix.jobs(4).await[0].state, "failed");
    fix.poll(vec![seen(4, false, Some("ccc"))], vec![], 2_000, DEBOUNCE)
        .await;
    let jobs = fix.jobs(4).await;
    assert_eq!(jobs.len(), 2);
    assert_eq!(jobs[1].state, "queued");
    assert_eq!((jobs[1].retry, jobs[1].next), (0, Some(62_000)));
    let moved = fix.target(4).await.unwrap();
    assert_eq!(
        (moved.head.as_deref(), moved.checked),
        (Some("ccc"), Some(2_000))
    );
}
