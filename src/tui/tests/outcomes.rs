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
        ("unknown", Some("failed"), "reconciliation failed"),
        ("unknown", Some("succeeded"), "marker not found"),
        ("submitted", None, "publication: submitted"),
    ] {
        store.call(move |conn| {
            conn.execute("UPDATE publications SET state=?1, reason=CASE WHEN ?1='unknown' THEN 'HTTP 503: service unavailable' END", [state])?;
            conn.execute("UPDATE jobs SET state='failed' WHERE kind='publish'", [])?;
            if let Some(result) = reconciliation {
                conn.execute("INSERT INTO jobs(kind, target_id, state, publication_id, failure_reason, enqueued_by_pid, enqueued_by_started_at, created_at, updated_at)
                SELECT 'reconcile',target_id,?1,id,'sensitive legacy stderr',1,0,2,2 FROM publications", [result])?;
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
        if state == "unknown" {
            assert!(
                text.contains("unknown: HTTP 503: service unavailable"),
                "{text}"
            );
        }
        if reconciliation == Some("failed") {
            assert!(!text.contains("marker not found"));
            assert!(!text.contains("sensitive legacy stderr"));
        }
    }
}

#[tokio::test]
async fn post_failures_remain_visible_on_the_queue_and_draft_after_reopening() {
    for (stderr, state, reason) in [
        (
            "gh: sensitive body (HTTP 422)",
            "rejected",
            "HTTP 422: validation failed",
        ),
        (
            "gh: sensitive body (HTTP 403)",
            "rejected",
            "HTTP 403: forbidden",
        ),
        (
            "gh: sensitive body (HTTP 503)",
            "unknown",
            "HTTP 503: service unavailable",
        ),
        (
            "gh: sensitive body (HTTP 429)",
            "unknown",
            "HTTP 429: too many requests",
        ),
        (
            "gh: sensitive body (HTTP 408)",
            "unknown",
            "HTTP 408: request timeout",
        ),
        (
            "Post /sensitive/422/reviews: unexpected EOF",
            "unknown",
            "no confirmed response",
        ),
        (
            "Post /sensitive/reviews: i/o timeout",
            "unknown",
            "request timed out",
        ),
    ] {
        let (tmp, store) = open().await;
        super::publish::seed(&store).await;
        let paths = paths(&tmp);
        let mut gh = super::fake::Fake::new(HEAD);
        gh.post_error = Some(stderr);
        let draft =
            super::publish::press(&store, &paths, &gh, &[KeyCode::Enter, KeyCode::Char('c')]).await;
        let expected = format!("{state}: {reason}");
        assert!(draft.contains(&expected), "{draft}");
        let stored = fixture::cell(
            &store,
            "SELECT failure_reason FROM jobs WHERE kind='publish'",
        )
        .await;
        assert_eq!(stored, reason);
        drop(store);

        let reopened = crate::store::Store::open(&paths).await.unwrap();
        assert_eq!(
            reopened.queue_rows().await.unwrap()[0].publication,
            expected
        );
        for keys in [&[][..], &[KeyCode::Enter][..]] {
            let text = super::publish::press(&reopened, &paths, &gh, keys).await;
            assert!(text.contains(&expected), "{text}");
            assert!(!text.contains("sensitive"), "{text}");
        }
        assert_eq!(gh.seen().len(), 1);
    }
}
