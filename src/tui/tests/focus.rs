use super::draft::{frame, marked, seed_draft};
use super::queue::{count, open, seed};
use ratatui::crossterm::event::KeyCode;

#[tokio::test]
async fn future_notes_can_be_focused_and_quit_writes_nothing() {
    let (_tmp, store) = open().await;
    seed(&store, 1).await;
    seed_draft(&store).await;
    let letters = [
        KeyCode::Enter,
        KeyCode::Char('k'),
        KeyCode::Char('d'),
        KeyCode::Char('e'),
    ];
    let (ignored, _) = frame(&store, &letters, 12).await;
    assert!(
        ignored.contains("undecided") && !ignored.contains("kept"),
        "{ignored}"
    );
    let limit = [KeyCode::Enter, KeyCode::Down, KeyCode::Down];
    let (_, focused) = frame(&store, &limit, 12).await;
    assert!(marked(&focused, "Limitation"), "{focused:?}");
    let down = [KeyCode::Enter, KeyCode::Down, KeyCode::Down, KeyCode::Down];
    let (_, focused) = frame(&store, &down, 12).await;
    assert!(marked(&focused, "note one"), "{focused:?}");
    assert!(!marked(&focused, "Limitation"), "{focused:?}");
    let jobs = count(&store, "SELECT count(*) FROM jobs").await;
    let publications = count(&store, "SELECT count(*) FROM publications").await;
    frame(&store, &[KeyCode::Enter, KeyCode::Char('q')], 12).await;
    assert_eq!(count(&store, "SELECT count(*) FROM jobs").await, jobs);
    assert_eq!(
        count(&store, "SELECT count(*) FROM publications").await,
        publications
    );
}
