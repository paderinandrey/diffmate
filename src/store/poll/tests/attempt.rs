use super::fixture::{Fix, seen};

const DEBOUNCE: i64 = 60_000;

#[tokio::test]
async fn a_different_head_before_the_first_run_resets_that_job() {
    let fix = Fix::open().await;
    fix.seed(1, true, true).await;
    fix.poll(vec![seen(1, false, Some("aaa"))], vec![], 1_000, DEBOUNCE)
        .await;
    let first = fix.jobs(1).await[0].id;
    fix.poll(vec![seen(1, false, Some("bbb"))], vec![], 2_000, DEBOUNCE)
        .await;
    let jobs = fix.jobs(1).await;
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, first);
    assert_eq!((jobs[0].retry, jobs[0].next), (0, Some(62_000)));
    assert_eq!(
        fix.target(1).await.unwrap().queued_head.as_deref(),
        Some("bbb")
    );
    assert_eq!(head_sha(&fix, first).await.as_deref(), Some("bbb"));
}

#[tokio::test]
async fn a_failed_attempt_without_a_run_is_not_queued_again() {
    let fix = Fix::open().await;
    fix.seed(1, true, true).await;
    fix.poll(vec![seen(1, false, Some("aaa"))], vec![], 1_000, DEBOUNCE)
        .await;
    let job = fix.jobs(1).await[0].id;
    assert_eq!(head_sha(&fix, job).await.as_deref(), Some("aaa"));
    fix.set_job(job, "failed", 0, None).await;
    fix.poll(vec![seen(1, false, Some("aaa"))], vec![], 2_000, DEBOUNCE)
        .await;
    assert_eq!(fix.jobs(1).await.len(), 1);
    fix.poll(vec![seen(1, false, Some("bbb"))], vec![], 3_000, DEBOUNCE)
        .await;
    let jobs = fix.jobs(1).await;
    assert_eq!(jobs.len(), 2);
    assert_eq!(jobs[1].state, "queued");
    assert_eq!(head_sha(&fix, jobs[1].id).await.as_deref(), Some("bbb"));
    fix.poll(vec![seen(1, false, Some("aaa"))], vec![], 4_000, DEBOUNCE)
        .await;
    assert!(fix.jobs(1).await.iter().all(|job| job.state != "queued"));
}

#[tokio::test]
async fn a_retry_stays_when_the_run_head_differs_from_the_queued_head() {
    let fix = Fix::open().await;
    fix.seed(1, true, true).await;
    fix.poll(vec![seen(1, false, Some("aaa"))], vec![], 1_000, DEBOUNCE)
        .await;
    let job = fix.jobs(1).await[0].id;
    fix.add_run(job, "bbb", true).await;
    fix.set_job(job, "queued", 1, Some(5_000)).await;
    assert_eq!(
        fix.target(1).await.unwrap().queued_head.as_deref(),
        Some("aaa")
    );
    fix.poll(vec![seen(1, false, Some("bbb"))], vec![], 2_000, DEBOUNCE)
        .await;
    let jobs = fix.jobs(1).await;
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, job);
    assert_eq!(jobs[0].state, "queued");
    assert_eq!((jobs[0].retry, jobs[0].next), (1, Some(5_000)));
}

async fn head_sha(fix: &Fix, job: i64) -> Option<String> {
    fix.store
        .call(move |conn| {
            conn.query_row("SELECT head_sha FROM jobs WHERE id = ?1", [job], |row| {
                row.get(0)
            })
        })
        .await
        .unwrap()
}
