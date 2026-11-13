use super::queue::key;
use ratatui::crossterm::event::KeyCode;

#[tokio::test]
async fn a_tick_keeps_the_service_running() {
    let (tx, mut keys) = tokio::sync::mpsc::channel(2);
    let gate = std::sync::Arc::new(tokio::sync::Notify::new());
    let wait = std::sync::Arc::clone(&gate);
    let work = async move {
        wait.notified().await;
        Ok(())
    };
    tx.send(Ok(None)).await.unwrap();
    tx.send(Ok(Some(key(KeyCode::Char('j'))))).await.unwrap();
    let mut turn = std::pin::pin!(super::super::finish_or_quit(work, &mut keys));
    for _ in 0..20 {
        tokio::select! {
            biased;
            ended = &mut turn => panic!("service ended on a tick: {ended:?}"),
            () = tokio::task::yield_now() => {}
        }
    }
    gate.notify_one();
    let super::super::Turn::Finished(queued) = turn.await.unwrap() else {
        panic!("quit on a tick");
    };
    assert_eq!(queued.len(), 1);
    assert!(matches!(queued[0].code, KeyCode::Char('j')));
}
