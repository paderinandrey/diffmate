use super::fixture::Fix;

#[tokio::test]
async fn enqueue_review_does_not_change_tracking_or_request_seen() {
    let fix = Fix::open().await;
    fix.seed(1, true, true).await;
    fix.seed(2, true, false).await;
    fix.enqueue(1, false).await;
    fix.enqueue(1, true).await;
    fix.enqueue(2, true).await;
    fix.enqueue(3, false).await;
    fix.enqueue(3, true).await;
    expect(&fix, 1, "tracked", true).await;
    expect(&fix, 2, "tracked", false).await;
    expect(&fix, 3, "untracked", false).await;
}

async fn expect(fix: &Fix, number: u64, tracking: &str, seen_bit: bool) {
    let row = fix.target(number).await.unwrap();
    assert_eq!((row.tracking.as_str(), row.seen), (tracking, seen_bit));
}
