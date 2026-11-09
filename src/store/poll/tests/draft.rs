use super::fixture::{Fix, seen};

const DEBOUNCE: i64 = 60_000;

#[tokio::test]
async fn a_draft_holds_a_queued_job_until_the_pr_is_ready() {
    let fix = Fix::open().await;
    fix.seed(1, true, true).await;
    fix.seed(2, true, true).await;
    fix.poll(vec![seen(1, false, Some("aaa"))], vec![], 1_000, DEBOUNCE)
        .await;
    let retry = fix.enqueue(2, true).await;
    fix.add_run(retry, "aaa", true).await;
    fix.set_job(retry, "queued", 1, Some(5_000)).await;
    fix.set_heads(2, Some("aaa"), None, None).await;
    fix.poll(
        vec![seen(1, true, None), seen(2, true, None)],
        vec![],
        70_000,
        DEBOUNCE,
    )
    .await;
    assert!(fix.store.next_runnable_job(70_000).await.unwrap().is_none());
    fix.poll(
        vec![seen(1, false, Some("aaa")), seen(2, false, Some("aaa"))],
        vec![],
        70_000,
        DEBOUNCE,
    )
    .await;
    let first = fix.jobs(1).await;
    assert_eq!(first.len(), 1);
    assert_eq!((first[0].retry, first[0].next), (0, Some(61_000)));
    let second = fix.jobs(2).await;
    assert_eq!(second.len(), 1);
    assert_eq!(
        (second[0].id, second[0].retry, second[0].next),
        (retry, 1, Some(5_000))
    );
    assert!(fix.store.next_runnable_job(70_000).await.unwrap().is_some());
}

#[tokio::test]
async fn a_follow_up_skips_an_untracked_target() {
    let fix = Fix::open().await;
    fix.seed(1, true, true).await;
    let job = fix.enqueue(1, true).await;
    fix.add_run(job, "aaa", false).await;
    fix.set_job(job, "succeeded", 0, None).await;
    let id = fix.id(1).await;
    fix.store.set_known_head(id, "bbb").await.unwrap();
    fix.store
        .call(move |conn| {
            conn.execute(
                "UPDATE targets SET tracking = 'untracked' WHERE id = ?1",
                [id],
            )
            .map(drop)
        })
        .await
        .unwrap();
    fix.after(1, 1_000, DEBOUNCE).await;
    assert_eq!(fix.jobs(1).await.len(), 1);
    assert_eq!(fix.target(1).await.unwrap().queued_head, None);
}
