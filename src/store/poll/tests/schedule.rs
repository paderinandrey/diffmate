use super::fixture::{Fix, seen};

const DEBOUNCE: i64 = 60_000;

#[tokio::test]
async fn a_new_head_is_armed_once_and_the_same_head_waits() {
    let fix = Fix::open().await;
    fix.seed(1, true, true).await;
    fix.set_heads(1, None, Some("old"), Some(4)).await;
    fix.poll(vec![seen(1, false, Some("aaa"))], vec![], 1_000, DEBOUNCE)
        .await;
    let jobs = fix.jobs(1).await;
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].state, "queued");
    assert_eq!(jobs[0].origin, "automatic");
    assert!(!jobs[0].force);
    assert_eq!((jobs[0].retry, jobs[0].next), (0, Some(61_000)));
    let row = fix.target(1).await.unwrap();
    assert_eq!(row.queued_head.as_deref(), Some("aaa"));
    assert_eq!((row.head.as_deref(), row.checked), (Some("old"), Some(4)));
    let id = jobs[0].id;
    fix.poll(vec![seen(1, false, Some("aaa"))], vec![], 9_000, DEBOUNCE)
        .await;
    let again = fix.jobs(1).await;
    assert_eq!(again.len(), 1);
    assert_eq!(again[0].id, id);
    assert_eq!((again[0].retry, again[0].next), (0, Some(61_000)));
    assert_eq!(fix.target(1).await.unwrap().head.as_deref(), Some("old"));
}

#[tokio::test]
async fn a_different_head_on_a_started_job_gets_its_own_attempt() {
    let fix = Fix::open().await;
    fix.seed(1, true, true).await;
    let job = fix.enqueue(1, true).await;
    fix.set_job(job, "queued", 1, Some(5_000)).await;
    fix.add_run(job, "aaa", true).await;
    fix.set_heads(1, Some("aaa"), Some("stored"), Some(8)).await;
    fix.poll(vec![seen(1, false, Some("bbb"))], vec![], 2_000, DEBOUNCE)
        .await;
    let jobs = fix.jobs(1).await;
    assert_eq!(jobs.len(), 2);
    assert_eq!(jobs[0].id, job);
    assert_eq!(jobs[0].state, "cancelled");
    assert_eq!((jobs[0].retry, jobs[0].next), (1, Some(5_000)));
    assert_eq!((jobs[1].retry, jobs[1].next), (0, Some(62_000)));
    assert!(
        fix.store
            .retry_revision(jobs[1].id)
            .await
            .unwrap()
            .is_none()
    );
    let row = fix.target(1).await.unwrap();
    assert_eq!(row.queued_head.as_deref(), Some("bbb"));
    assert_eq!(
        (row.head.as_deref(), row.checked),
        (Some("stored"), Some(8))
    );
    fix.poll(vec![seen(1, false, Some("bbb"))], vec![], 3_000, DEBOUNCE)
        .await;
    let again = fix.jobs(1).await;
    assert_eq!(again.len(), 2);
    assert_eq!(
        (again[1].id, again[1].retry, again[1].next),
        (jobs[1].id, 0, Some(62_000))
    );
}

#[tokio::test]
async fn the_same_head_keeps_a_retry_deadline() {
    let fix = Fix::open().await;
    fix.seed(1, true, true).await;
    fix.seed(2, true, true).await;
    let from_run = fix.enqueue(1, true).await;
    fix.set_job(from_run, "queued", 1, Some(5_000)).await;
    fix.add_run(from_run, "aaa", true).await;
    fix.set_heads(1, None, None, None).await;
    let armed = fix.enqueue(2, true).await;
    fix.set_job(armed, "queued", 3, Some(7_000)).await;
    fix.set_heads(2, Some("aaa"), Some("kept"), Some(2)).await;
    fix.poll(
        vec![seen(1, false, Some("aaa")), seen(2, false, Some("aaa"))],
        vec![],
        2_000,
        DEBOUNCE,
    )
    .await;
    let first = &fix.jobs(1).await[0];
    assert_eq!((first.retry, first.next), (1, Some(5_000)));
    assert_eq!(fix.target(1).await.unwrap().queued_head, None);
    let second = &fix.jobs(2).await[0];
    assert_eq!(
        (second.id, second.retry, second.next),
        (armed, 3, Some(7_000))
    );
    assert_eq!(fix.target(2).await.unwrap().head.as_deref(), Some("kept"));
}
