use super::fixture::{Fix, absent_open, closed, pr, seen};

const DEBOUNCE: i64 = 60_000;

#[tokio::test]
async fn a_closed_pr_cancels_queued_automatic_work() {
    let fix = Fix::open().await;
    fix.seed(1, true, true).await;
    fix.seed(2, true, true).await;
    fix.seed(3, true, true).await;
    fix.seed(4, true, true).await;
    let retry = fix.enqueue(1, true).await;
    fix.add_run(retry, "aaa", true).await;
    fix.set_job(retry, "queued", 1, Some(5_000)).await;
    fix.store
        .call(move |conn| {
            conn.execute("UPDATE jobs SET held = 1 WHERE id = ?1", [retry])
                .map(drop)
        })
        .await
        .unwrap();
    let running = fix.enqueue(2, true).await;
    fix.store.start_job(running).await.unwrap();
    let manual = fix.enqueue(3, false).await;
    let open = fix.enqueue(4, true).await;
    fix.poll(
        vec![],
        vec![
            (pr(1), closed(None)),
            (pr(2), closed(None)),
            (pr(3), closed(None)),
            (pr(4), absent_open(false, Some("bbb"))),
        ],
        9_000,
        DEBOUNCE,
    )
    .await;
    let closed = fix.jobs(1).await;
    assert_eq!(closed.len(), 1);
    assert_eq!(
        (closed[0].id, closed[0].state.as_str()),
        (retry, "cancelled")
    );
    assert_eq!(fix.target(1).await.unwrap().tracking, "untracked");
    assert_eq!(runs(&fix, retry).await, 1);
    let live = &fix.jobs(2).await[0];
    assert_eq!((live.id, live.state.as_str()), (running, "running"));
    assert_eq!(fix.target(2).await.unwrap().tracking, "untracked");
    let kept = &fix.jobs(3).await[0];
    assert_eq!(
        (kept.id, kept.state.as_str(), kept.origin.as_str()),
        (manual, "queued", "manual")
    );
    let still = &fix.jobs(4).await[0];
    assert_eq!((still.id, still.state.as_str()), (open, "queued"));
    assert_eq!(fix.target(4).await.unwrap().tracking, "tracked");
}

#[tokio::test]
async fn a_closed_target_tracks_again_and_a_discard_stays_latched() {
    let fix = Fix::open().await;
    fix.seed(1, true, true).await;
    fix.seed(2, false, true).await;
    fix.poll(
        vec![],
        vec![(pr(1), closed(Some("zzz"))), (pr(2), closed(None))],
        1_000,
        DEBOUNCE,
    )
    .await;
    let closed = fix.target(1).await.unwrap();
    assert_eq!(
        (closed.tracking.as_str(), closed.seen),
        ("untracked", false)
    );
    assert_eq!(
        (closed.head.as_deref(), closed.checked),
        (Some("zzz"), Some(1_000))
    );
    let discarded = fix.target(2).await.unwrap();
    assert_eq!(
        (discarded.tracking.as_str(), discarded.seen),
        ("untracked", true)
    );
    fix.poll(
        vec![seen(1, false, Some("aaa")), seen(2, false, Some("bbb"))],
        vec![],
        2_000,
        DEBOUNCE,
    )
    .await;
    let again = fix.target(1).await.unwrap();
    assert_eq!((again.tracking.as_str(), again.seen), ("tracked", true));
    assert_eq!(
        (again.head.as_deref(), again.checked),
        (Some("aaa"), Some(2_000))
    );
    assert_eq!(fix.jobs(1).await.len(), 1);
    let still = fix.target(2).await.unwrap();
    assert_eq!((still.tracking.as_str(), still.seen), ("untracked", true));
    assert!(fix.jobs(2).await.is_empty());
}

async fn runs(fix: &Fix, job: i64) -> i64 {
    fix.store
        .call(move |conn| {
            conn.query_row(
                "SELECT count(*) FROM runs WHERE job_id = ?1",
                [job],
                |row| row.get(0),
            )
        })
        .await
        .unwrap()
}
