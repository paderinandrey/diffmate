use super::fixture::Fix;

const DEBOUNCE: i64 = 60_000;

#[tokio::test]
async fn a_waiting_retry_for_the_stored_head_keeps_its_deadline() {
    let fix = Fix::open().await;
    fix.seed(1, true, true).await;
    let job = fix.enqueue(1, true).await;
    fix.add_run(job, "aaa", true).await;
    fix.set_job(job, "queued", 1, Some(5_000)).await;
    fix.set_heads(1, None, Some("aaa"), Some(6)).await;
    fix.after(1, 9_000, DEBOUNCE).await;
    let row = &fix.jobs(1).await[0];
    assert_eq!((row.id, row.retry, row.next), (job, 1, Some(5_000)));
    assert_eq!(fix.target(1).await.unwrap().queued_head, None);
    assert_eq!(fix.target(1).await.unwrap().checked, Some(6));
}
