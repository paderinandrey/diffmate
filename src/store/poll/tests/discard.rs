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
    fix.poll(
        vec![],
        vec![(pr(1), absent_open(false, Some("bbb")))],
        2_000,
        60_000,
    )
    .await;
    expect(&fix, 1, "untracked", false).await;
    fix.poll(vec![seen(1, false, Some("ccc"))], vec![], 3_000, 60_000)
        .await;
    expect(&fix, 1, "tracked", true).await;
    assert_eq!(fix.store.known_targets().await.unwrap().len(), 1);
}

#[tokio::test]
async fn discard_deletes_only_an_unstarted_automatic_job() {
    let fix = Fix::open().await;
    fix.enqueue(1, true).await;
    let running = fix.enqueue(2, true).await;
    fix.store.start_job(running).await.unwrap();
    let reviewed = fix.enqueue(3, true).await;
    fix.add_run(reviewed, "aaa", false).await;
    let retrying = fix.enqueue(4, true).await;
    fix.set_job(retrying, "queued", 2, Some(50)).await;
    let manual = fix.enqueue(5, false).await;
    for number in 1..=5 {
        fix.store.discard(&pr(number)).await.unwrap();
        expect(&fix, number, "untracked", true).await;
    }
    assert!(fix.jobs(1).await.is_empty());
    let running_jobs = fix.jobs(2).await;
    assert_eq!(running_jobs[0].origin, "automatic");
    assert!(!running_jobs[0].force);
    assert_eq!(states(&running_jobs), vec![("running", running)]);
    assert_eq!(states(&fix.jobs(3).await), vec![("queued", reviewed)]);
    assert_eq!(fix.jobs(4).await[0].retry, 2);
    assert_eq!(fix.jobs(4).await[0].next, Some(50));
    assert_eq!(states(&fix.jobs(5).await), vec![("queued", manual)]);
}

fn states(jobs: &[Job]) -> Vec<(&str, i64)> {
    jobs.iter()
        .map(|job| (job.state.as_str(), job.id))
        .collect()
}

async fn expect(fix: &Fix, number: u64, tracking: &str, seen: bool) {
    let row = fix.target(number).await.unwrap();
    assert_eq!((row.tracking.as_str(), row.seen), (tracking, seen));
    assert_eq!((row.head, row.queued_head, row.checked), (None, None, None));
}
