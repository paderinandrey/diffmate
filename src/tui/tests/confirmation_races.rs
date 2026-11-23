use super::fixture::{self, cell, open_session};
use super::publish::paths;
use super::queue::{HEAD, key, open};
use crate::tui::{Captured, drive_queued, reader::KeyGate};
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use std::sync::Arc;

#[tokio::test]
async fn queued_navigation_and_late_delivery_cannot_accept_a_moved_head_prompt() {
    let (tmp, store) = open().await;
    fixture::plant(&store, "pending", "needs_confirmation").await;
    store
        .call(|conn| conn.execute("UPDATE jobs SET head_sha=?1 WHERE kind='publish'", [HEAD]))
        .await
        .unwrap();
    let paths = paths(&tmp);
    let mut session = open_session(&store, &paths.lock()).await;
    let gh = super::fake::Fake::new(HEAD);
    let git = crate::git::GitCli::new(&crate::procenv::InheritedEnv::from_pairs(Vec::new()));
    let settings = crate::config::Settings::default();
    let mode = fixture::mode(&gh, &git, &paths, &settings);
    let mut terminal = ratatui::Terminal::new(TestBackend::new(160, 8)).unwrap();
    super::super::draw_scripted(&mut terminal, &mut session);
    let buffered = [KeyCode::Down, KeyCode::Enter].map(|code| Captured::from(key(code)));
    let late = Captured::from(key(KeyCode::Enter));
    super::super::service(&mut session, &store, &mode)
        .await
        .unwrap();
    super::super::draw_scripted(&mut terminal, &mut session);
    assert!(session.confirming());
    drive_queued(&mut session, &store, &mode, buffered.into(), None)
        .await
        .unwrap();
    let gate = Arc::new(KeyGate::new());
    let (tx, mut rx) = tokio::sync::mpsc::channel(2);
    tx.send(Ok(Some(late))).await.unwrap();
    drive_queued(
        &mut session,
        &store,
        &mode,
        Vec::new(),
        Some((&gate, &mut rx)),
    )
    .await
    .unwrap();
    assert!(session.confirm_modal().unwrap().highlighted_safe);
    assert_eq!(
        cell(&store, "SELECT state FROM jobs WHERE kind='publish'").await,
        "needs_confirmation"
    );
    assert_eq!(gh.seen(), []);
    tx.send(Ok(Some(key(KeyCode::Enter).into()))).await.unwrap();
    drive_queued(
        &mut session,
        &store,
        &mode,
        Vec::new(),
        Some((&gate, &mut rx)),
    )
    .await
    .unwrap();
    assert_eq!(
        cell(&store, "SELECT state FROM publications").await,
        "abandoned"
    );
}
