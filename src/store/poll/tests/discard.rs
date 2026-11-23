use super::fixture::{Fix, absent_open, pr, seen};
use super::jobs::Job;

#[tokio::test]
async fn discard_hides_a_request_until_it_leaves_and_returns() {
    let fix = Fix::open().await;
    fix.store.discard(&pr(1)).await.unwrap();
    expect(&fix, 1, "untracked", true).await;
    fix.poll(vec![seen(1, false, Some("aaa"))], vec![], 1_000, 60_000)
        .await;
    expect(&fix, 1, "untracked", true).await;
    assert_eq!(fix.target(1).await.unwrap().head.as_deref(), Some("aaa"));
    fix.poll(
        vec![],
        vec![(pr(1), absent_open(false, Some("bbb")))],
        2_000,
        60_000,
    )
    .await;
    expect(&fix, 1, "untracked", false).await;
    assert_eq!(
        (
            fix.target(1).await.unwrap().head.as_deref(),
            fix.target(1).await.unwrap().checked
        ),
        (Some("bbb"), Some(2_000))
    );
    fix.poll(vec![seen(1, false, Some("ccc"))], vec![], 3_000, 60_000)
        .await;
    let row = fix.target(1).await.unwrap();
    assert_eq!((row.tracking.as_str(), row.seen), ("tracked", true));
    assert_eq!(row.queued_head.as_deref(), Some("ccc"));
    assert_eq!(
        (row.head.as_deref(), row.checked),
        (Some("ccc"), Some(3_000))
    );
    assert_eq!(fix.store.poll_targets().await.unwrap().len(), 1);
}

#[tokio::test]
async fn discard_cancels_waiting_automatic_jobs_without_losing_their_history() {
    let fix = Fix::open().await;
    let waiting = fix.enqueue(1, true).await;
    let attached = fix.enqueue(1, false).await;
    assert_eq!(attached, waiting);
    let running = fix.enqueue(2, true).await;
    fix.store.start_job(running).await.unwrap();
    let reviewed = fix.enqueue(3, true).await;
    fix.add_run(reviewed, "aaa", true).await;
    let retrying = fix.enqueue(4, true).await;
    fix.set_job(retrying, "queued", 2, Some(50)).await;
    let manual = fix.enqueue(5, false).await;
    for number in 1..=5 {
        fix.store.discard(&pr(number)).await.unwrap();
        expect(&fix, number, "untracked", true).await;
    }
    let result = fix.store.stored_result(attached).await.unwrap();
    assert_eq!(result.state, "cancelled");
    assert_eq!(result.reason.as_deref(), Some("target discarded"));
    let running_jobs = fix.jobs(2).await;
    assert_eq!(running_jobs[0].origin, "automatic");
    assert!(!running_jobs[0].force);
    assert_eq!(states(&running_jobs), vec![("running", running)]);
    assert_eq!(states(&fix.jobs(3).await), vec![("cancelled", reviewed)]);
    assert!(
        fix.store
            .stored_result(reviewed)
            .await
            .unwrap()
            .run
            .is_some()
    );
    assert_eq!(states(&fix.jobs(4).await), vec![("cancelled", retrying)]);
    assert_eq!(fix.jobs(4).await[0].retry, 2);
    assert_eq!(fix.jobs(4).await[0].next, None);
    assert_eq!(states(&fix.jobs(5).await), vec![("queued", manual)]);
}

#[tokio::test]
async fn recovery_does_not_requeue_a_running_attempt_after_discard() {
    let fix = Fix::open().await;
    let job = fix.enqueue(1, true).await;
    fix.set_job(job, "running", 1, None).await;
    fix.store.discard(&pr(1)).await.unwrap();
    let stopped = fix.store.resolve_running(3, 100).await.unwrap();
    assert_eq!(stopped.jobs, [(job, "failed (target discarded)".into())]);
    assert_eq!(fix.jobs(1).await[0].retry, 1);
    assert!(
        fix.store
            .next_runnable_job(i64::MAX)
            .await
            .unwrap()
            .is_none()
    );
}

fn states(jobs: &[Job]) -> Vec<(&str, i64)> {
    jobs.iter()
        .map(|job| (job.state.as_str(), job.id))
        .collect()
}

async fn expect(fix: &Fix, number: u64, tracking: &str, seen: bool) {
    let row = fix.target(number).await.unwrap();
    assert_eq!((row.tracking.as_str(), row.seen), (tracking, seen));
    assert_eq!(row.queued_head, None);
}
