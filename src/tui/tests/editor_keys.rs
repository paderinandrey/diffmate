use super::queue::key;
use crate::error::Error;
use ratatui::crossterm::event::KeyCode;
use std::sync::{Arc, mpsc};

#[tokio::test]
async fn editor_keys_are_not_replayed_as_commands() {
    let (tx, mut keys) = tokio::sync::mpsc::channel(4);
    let gate = Arc::new(super::super::reader::KeyGate::new());
    let (slot_tx, slot_rx) = mpsc::channel();
    let (entered_tx, entered_rx) = mpsc::channel();
    let pump_tx = tx.clone();
    let reader_gate = Arc::clone(&gate);
    let reader = std::thread::spawn(move || {
        let mut primed = false;
        super::super::pump(&pump_tx, &reader_gate, move || {
            if !primed {
                primed = true;
                let _ = entered_tx.send(());
            }
            slot_rx.recv().unwrap()
        });
    });
    entered_rx.recv().unwrap();
    tx.send(Ok(Some(key(KeyCode::Char('c'))))).await.unwrap();
    let release = slot_tx.clone();
    let waiting = Arc::clone(&gate);
    std::thread::spawn(move || {
        while !waiting.suspend_requested() {
            std::thread::yield_now();
        }
        release.send(Ok(Some(key(KeyCode::Char('a'))))).unwrap();
    });
    let seen = Arc::clone(&gate);
    super::super::pause_for_editor(&gate, &mut keys, async move {
        assert!(seen.is_paused());
        Ok::<(), Error>(())
    })
    .await
    .unwrap();
    assert!(!gate.is_paused());
    slot_tx.send(Ok(Some(key(KeyCode::Char('j'))))).unwrap();
    let got = keys.recv().await.unwrap().unwrap().unwrap();
    assert!(matches!(got.code, KeyCode::Char('j')));
    assert!(keys.try_recv().is_err());
    slot_tx
        .send(Err(Error::io("read a key")(std::io::Error::other("stop"))))
        .unwrap();
    reader.join().unwrap();
}
