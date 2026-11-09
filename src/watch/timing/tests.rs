use super::idle_delay;
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn idle_delay_is_the_soonest_of_the_cap_poll_and_attempt() {
    assert_eq!(idle_delay(0, 60_000, None), Duration::from_secs(1));
    assert_eq!(idle_delay(0, 100, None), Duration::from_millis(100));
    assert_eq!(idle_delay(0, 60_000, Some(200)), Duration::from_millis(200));
}

#[tokio::test(start_paused = true)]
async fn a_nearer_attempt_wakes_before_the_poll_on_a_paused_clock() {
    let delay = idle_delay(0, 60_000, Some(1_500));
    assert_eq!(delay, Duration::from_secs(1));
    let handle = tokio::spawn(async move {
        tokio::time::sleep(delay).await;
    });
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(999)).await;
    assert!(!handle.is_finished());
    tokio::time::advance(Duration::from_millis(1)).await;
    handle.await.unwrap();
}
