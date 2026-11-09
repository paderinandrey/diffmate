use super::fixture::Fix;

const DEBOUNCE: i64 = 60_000;

#[tokio::test]
async fn a_head_stored_after_success_gets_one_follow_up_job() {
    let fix = Fix::open().await;
    fix.seed(1, true, true).await;
    let job = fix.enqueue(1, true).await;
    fix.add_run(job, "aaa", false).await;
    fix.set_job(job, "succeeded", 0, None).await;
    let id = fix.id(1).await;
    fix.store.set_known_head(id, "bbb").await.unwrap();
    let before = fix.target(1).await.unwrap();
    fix.after(1, 1_000, DEBOUNCE).await;
    let after = fix.target(1).await.unwrap();
    assert_eq!(after.head, before.head);
    assert_eq!(after.checked, before.checked);
    assert_eq!(after.queued_head.as_deref(), Some("bbb"));
    let jobs = fix.jobs(1).await;
    assert_eq!(jobs.len(), 2);
    assert_eq!(jobs[0].state, "succeeded");
    assert_eq!(jobs[1].origin, "automatic");
    assert!(!jobs[1].force);
    assert_eq!((jobs[1].retry, jobs[1].next), (0, Some(61_000)));
    let state: String = fix
        .store
        .call(move |conn| {
            conn.query_row("SELECT state FROM runs WHERE job_id = ?1", [job], |row| {
                row.get(0)
            })
        })
        .await
        .unwrap();
    assert_eq!(state, "succeeded");
}

#[tokio::test]
async fn schedule_after_run_holds_resets_or_blocks() {
    let fix = Fix::open().await;
    fix.seed(1, true, true).await;
    fix.seed(2, true, true).await;
    fix.seed(3, true, true).await;
    fix.seed(4, true, true).await;
    fix.seed(5, true, true).await;
    let same = fix.enqueue(1, true).await;
    fix.add_run(same, "aaa", false).await;
    fix.set_job(same, "succeeded", 0, None).await;
    fix.set_heads(1, None, Some("aaa"), Some(1)).await;
    let blocked = fix.enqueue(2, true).await;
    fix.add_run(blocked, "aaa", false).await;
    fix.set_job(blocked, "succeeded", 0, None).await;
    let extra = fix.enqueue(2, true).await;
    fix.set_heads(2, None, Some("bbb"), Some(2)).await;
    let reset = fix.enqueue(3, true).await;
    fix.add_run(reset, "aaa", true).await;
    fix.set_job(reset, "queued", 1, Some(4_000)).await;
    fix.set_heads(3, None, Some("bbb"), Some(3)).await;
    let running = fix.enqueue(4, true).await;
    fix.store.start_job(running).await.unwrap();
    fix.set_heads(4, Some("old"), Some("bbb"), Some(4)).await;
    fix.after(1, 1_000, DEBOUNCE).await;
    fix.after(2, 1_000, DEBOUNCE).await;
    fix.after(3, 2_000, DEBOUNCE).await;
    fix.after(4, 2_000, DEBOUNCE).await;
    fix.after(5, 2_000, DEBOUNCE).await;
    assert_eq!(fix.jobs(1).await.len(), 1);
    let queued = fix.jobs(2).await;
    assert_eq!(queued.len(), 2);
    assert_eq!(queued[1].id, extra);
    assert_eq!(queued[1].next, None);
    let retried = fix.jobs(3).await;
    assert_eq!(retried.len(), 2);
    assert_eq!(retried[0].state, "cancelled");
    assert_eq!((retried[0].id, retried[0].retry), (reset, 1));
    assert_eq!((retried[1].retry, retried[1].next), (0, Some(62_000)));
    assert_eq!(
        fix.target(3).await.unwrap().queued_head.as_deref(),
        Some("bbb")
    );
    assert_eq!(fix.target(3).await.unwrap().checked, Some(3));
    let live = &fix.jobs(4).await[0];
    assert_eq!(live.state, "running");
    assert_eq!(
        fix.target(4).await.unwrap().queued_head.as_deref(),
        Some("old")
    );
    assert!(fix.jobs(5).await.is_empty());
}
