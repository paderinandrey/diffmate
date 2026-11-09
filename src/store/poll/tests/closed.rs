use super::super::Absent;
use super::fixture::{Fix, absent_open, pr};

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
            (pr(1), Absent::Closed),
            (pr(2), Absent::Closed),
            (pr(3), Absent::Closed),
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
