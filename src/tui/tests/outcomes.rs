use super::fixture::{self, plant};
use super::publish::paths;
use super::queue::{HEAD, key, open};
use crate::config::Settings;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;

#[tokio::test]
async fn the_open_draft_shows_external_outcomes_and_reconciliation_evidence() {
    let (tmp, store) = open().await;
    plant(&store, "pending", "queued").await;
    let paths = paths(&tmp);
    let _other_runner = super::publish::hold(&paths.lock());
    let mut session = fixture::open_session(&store, &paths.lock()).await;
    let gh = super::fake::Fake::new(HEAD);
    let git = crate::git::GitCli::new(&crate::procenv::InheritedEnv::from_pairs(Vec::new()));
    let settings = Settings::default();
    let mode = fixture::mode(&gh, &git, &paths, &settings);
    super::super::apply(&mut session, &store, &mode, key(KeyCode::Enter), None)
        .await
        .unwrap();
    let mut terminal = ratatui::Terminal::new(TestBackend::new(160, 8)).unwrap();
    for (state, reconciliation, expected) in [
        ("rejected", None, "publication: rejected"),
        ("unknown", None, "y: reconcile"),
        (
            "unknown",
            Some("failed"),
            "reconciliation failed: not authenticated",
        ),
        ("unknown", Some("succeeded"), "marker not found"),
        ("submitted", None, "publication: submitted"),
    ] {
        store.call(move |conn| {
            conn.execute("UPDATE publications SET state=?1", [state])?;
            conn.execute("UPDATE jobs SET state='failed' WHERE kind='publish'", [])?;
            if let Some(result) = reconciliation {
                conn.execute("INSERT INTO jobs(kind, target_id, state, publication_id, failure_reason, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at)
                SELECT 'reconcile',target_id,?1,id,'not authenticated',1,0,2,2 FROM publications", [result])?;
            }
            Ok(())
        }).await.unwrap();
        super::super::service(&mut session, &store, &mode)
            .await
            .unwrap();
        super::super::draw_scripted(&mut terminal, &mut session);
        let text = terminal.backend().to_string();
        assert!(session.on_draft());
        assert!(text.contains(expected), "{text}");
        if reconciliation == Some("failed") {
            assert!(!text.contains("marker not found"));
        }
    }
}
