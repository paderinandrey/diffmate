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

#[tokio::test]
async fn a_full_key_channel_still_opens_the_editor() {
    let (tx, mut keys) = tokio::sync::mpsc::channel(16);
    let gate = Arc::new(super::super::reader::KeyGate::new());
    let reader_gate = Arc::clone(&gate);
    let reader = std::thread::spawn(move || {
        super::super::pump(&tx, &reader_gate, || Ok(Some(key(KeyCode::Char('x')))));
    });
    let mut opened = false;
    {
        let mut edit = std::pin::pin!(super::super::pause_for_editor(&gate, &mut keys, async {
            assert!(gate.is_paused());
            Ok::<(), Error>(())
        }));
        for _ in 0..2_000 {
            tokio::select! {
                biased;
                ended = &mut edit => {
                    ended.unwrap();
                    opened = true;
                    break;
                }
                () = tokio::task::yield_now() => {}
            }
        }
    }
    gate.stop();
    drop(keys);
    reader.join().unwrap();
    assert!(opened, "editor did not open");
}

#[tokio::test]
async fn cancelled_editor_pause_wakes_the_reader() {
    let (tx, mut keys) = tokio::sync::mpsc::channel(4);
    let gate = Arc::new(super::super::reader::KeyGate::new());
    let (slot_tx, slot_rx) = mpsc::channel();
    let (entered_tx, entered_rx) = mpsc::channel();
    let reader_gate = Arc::clone(&gate);
    let reader = std::thread::spawn(move || {
        super::super::pump(&tx, &reader_gate, move || {
            let _ = entered_tx.send(());
            slot_rx.recv().unwrap()
        });
    });
    entered_rx.recv().unwrap();
    {
        let edit = super::super::pause_for_editor(&gate, &mut keys, std::future::pending::<()>());
        let mut edit = std::pin::pin!(edit);
        for _ in 0..2_000 {
            if gate.suspend_requested() {
                break;
            }
            tokio::select! {
                biased;
                () = &mut edit => panic!("pending edit ended"),
                () = tokio::task::yield_now() => {}
            }
        }
        assert!(gate.suspend_requested());
    }
    slot_tx.send(Ok(None)).unwrap();
    let mut finished = false;
    for _ in 0..2_000 {
        if reader.is_finished() {
            finished = true;
            break;
        }
        std::thread::yield_now();
        tokio::task::yield_now().await;
    }
    if !finished {
        gate.stop();
        let _ = slot_tx.send(Ok(None));
    }
    reader.join().unwrap();
    assert!(finished);
}
