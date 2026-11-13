use super::fake::Fake;
use super::fixture::{self, open_session};
use super::publish::paths;
use super::queue::{self, HEAD, key, open};
use crate::config::Settings;
use ratatui::crossterm::event::KeyCode;

#[tokio::test]
async fn a_stale_confirmation_publishes_its_own_draft() {
    let (tmp, store) = open().await;
    queue::seed(&store, 1).await;
    store
        .call(|conn| conn.execute("UPDATE findings SET decision = 'kept' WHERE id = '1'", []))
        .await
        .unwrap();
    let paths = paths(&tmp);
    let mut session = open_session(&store, &paths.lock()).await;
    session.open_draft(&store).await.unwrap();
    session.ask_stale(1, "COMMENT", HEAD, "stale".to_string());
    let head = HEAD;
    store
        .call(move |conn| {
            conn.execute_batch(&format!(
                "INSERT INTO jobs (id, kind, target_id, state, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at) VALUES (3, 'review', 1, 'succeeded', 1, 0, 3, 3);
                 INSERT INTO runs (id, target_id, job_id, state, result, head_sha, merge_base_sha, base_tip_sha, base_ref, created_at) VALUES (2, 1, 3, 'succeeded', 'no_findings', '{head}', 'base', 'base', 'main', 3);
                 INSERT INTO drafts (id, run_id, original_output, created_at) VALUES (2, 2, '{{}}', 3);"
            ))
        })
        .await
        .unwrap();
    let gh = Fake::new(HEAD);
    let git = crate::git::GitCli::new(&crate::procenv::InheritedEnv::from_pairs(Vec::new()));
    let settings = Settings::default();
    let mode = fixture::mode(&gh, &git, &paths, &settings);
    super::super::service(&mut session, &store, &mode)
        .await
        .unwrap();
    assert_eq!(session.draft().unwrap().0.id, 1);
    assert!(session.confirming());
    session.highlight_confirm(false);
    super::super::apply(&mut session, &store, &mode, key(KeyCode::Enter))
        .await
        .unwrap();
    assert_eq!(
        queue::count(&store, "SELECT draft_id FROM publications").await,
        1
    );
    assert_eq!(
        queue::count(
            &store,
            "SELECT count(*) FROM publications WHERE draft_id = 2"
        )
        .await,
        0
    );
    super::super::service(&mut session, &store, &mode)
        .await
        .unwrap();
    assert!(!session.confirming());
    assert_eq!(session.draft().unwrap().0.id, 2);
}
