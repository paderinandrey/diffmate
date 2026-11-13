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
    let mode = fixture::mode(&gh, &git, &paths, &settings);
    super::super::service(&mut session, &store, &mode)
        .await
        .unwrap();
    assert!(!session.running());
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
}
