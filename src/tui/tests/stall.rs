use super::fixture::{self, cell, plant};
use super::hang::Hang;
use super::publish::paths;
use super::queue::{key, open};
use crate::config::Settings;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

async fn quit_during_send(pressed: KeyEvent) {
    let (tmp, store) = open().await;
    plant(&store, "pending", "queued").await;
    let paths = paths(&tmp);
    let mut session = fixture::open_session(&store, &paths.lock()).await;
    let gh = Hang;
    let git = crate::git::GitCli::new(&crate::procenv::InheritedEnv::from_pairs(Vec::new()));
    let settings = Settings::default();
    let mode = fixture::mode(&gh, &git, &paths, &settings);
    let (tx, mut keys) = tokio::sync::mpsc::channel(4);
    let mut turn = std::pin::pin!(super::super::finish_or_quit(
        super::super::service(&mut session, &store, &mode),
        &mut keys,
    ));
    for _ in 0..1000 {
        if cell(&store, "SELECT state FROM publications").await == "sending" {
            break;
        }
        tokio::select! {
            biased;
            ended = &mut turn => panic!("publish finished: {ended:?}"),
            () = tokio::task::yield_now() => {}
        }
    }
    assert_eq!(
        cell(&store, "SELECT state FROM publications").await,
        "sending"
    );
    tx.send(Ok(Some(pressed))).await.unwrap();
    assert!(matches!(turn.await.unwrap(), super::super::Turn::Quit));
    assert_eq!(
        cell(&store, "SELECT state FROM publications").await,
        "sending"
    );
    assert_eq!(
        cell(&store, "SELECT state FROM jobs WHERE id = 2").await,
        "running"
    );
}

#[tokio::test]
async fn quitting_a_stalled_publish_leaves_it_sending() {
    quit_during_send(key(KeyCode::Char('q'))).await;
    quit_during_send(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)).await;
}
