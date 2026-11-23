use super::fixture::{self, cell, plant};
use super::publish::{hold, paths};
use super::queue::{HEAD, open};
use crate::config::Settings;

#[tokio::test]
async fn a_held_lock_defers_recovery_until_it_frees() {
    let (tmp, store) = open().await;
    plant(&store, "pending", "running").await;
    let paths = paths(&tmp);
    let held = hold(&paths.lock());
    let mut session = fixture::open_session(&store, &paths.lock()).await;
    let gh = super::fake::Fake::new(HEAD);
    let git = crate::git::GitCli::new(&crate::procenv::InheritedEnv::from_pairs(Vec::new()));
    let settings = Settings::default();
    let painted = std::cell::RefCell::new(Vec::new());
    let paint = |session: &mut crate::tui::model::Session, running| {
        let mut terminal =
            ratatui::Terminal::new(ratatui::backend::TestBackend::new(80, 8)).unwrap();
        terminal
            .draw(|frame| {
                crate::tui::draw::screen(frame, session, running);
            })
            .unwrap();
        painted
            .borrow_mut()
            .push(terminal.backend().to_string().contains("runner: tui"));
        Ok(())
    };
    let mode = crate::tui::post::Mode::On {
        gh: &gh,
        git: &git,
        paths: &paths,
        settings: &settings,
        paint_runner: Some(&paint),
    };
    super::super::service(&mut session, &store, &mode)
        .await
        .unwrap();
    assert!(
        crate::lock::try_acquire(&paths.lock())
            .await
            .unwrap()
            .is_none()
    );
    assert!(painted.borrow().is_empty());
    assert_eq!(
        cell(&store, "SELECT state FROM publications").await,
        "pending"
    );
    assert_eq!(
        cell(&store, "SELECT state FROM jobs WHERE id = 2").await,
        "running"
    );
    drop(held);
    super::super::service(&mut session, &store, &mode)
        .await
        .unwrap();
    assert_eq!(
        cell(&store, "SELECT state FROM publications").await,
        "submitted"
    );
    assert_eq!(*painted.borrow(), [true, false]);

    let next = crate::lock::try_acquire(&paths.lock())
        .await
        .unwrap()
        .unwrap();
    next.announce(crate::lock::Role::Review).await.unwrap();
    assert_eq!(
        crate::lock::holder(&paths.lock()).await,
        Some(crate::lock::Role::Review)
    );

    store
        .call(|conn| conn.execute_batch("UPDATE jobs SET state = 'running' WHERE kind = 'sync'"))
        .await
        .unwrap();
    drop(next);
    super::super::service(&mut session, &store, &mode)
        .await
        .unwrap();
    assert_eq!(*painted.borrow(), [true, false, true, false]);
    assert_ne!(
        cell(&store, "SELECT state FROM jobs WHERE kind = 'sync'").await,
        "running"
    );
}
