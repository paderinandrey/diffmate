use super::queue::{self, open, seed};
use crate::tui::model::Session;
use crate::tui::spawn::{Browser, Editor, ExitFuture, OpenFuture};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::ExitStatus;
use std::sync::{Arc, Mutex};

struct Rewrite {
    mode: Arc<Mutex<u32>>,
}

impl Editor for Rewrite {
    fn launch<'a>(&'a self, argv: &'a [String], path: &'a Path) -> ExitFuture<'a> {
        Box::pin(async move {
            assert_eq!(argv.len(), 1);
            assert_eq!(argv[0], "ed");
            let mode = std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
            *self.mode.lock().unwrap() = mode;
            std::fs::write(path, b"Kept\n\nfresh body\n").unwrap();
            Ok(ExitStatus::from_raw(0))
        })
    }
}

struct Recorded {
    calls: Arc<Mutex<Vec<(String, String)>>>,
}

impl Browser for Recorded {
    fn launch<'a>(&'a self, program: &'a str, url: &'a str) -> OpenFuture<'a> {
        let call = (program.to_owned(), url.to_owned());
        Box::pin(async move {
            self.calls.lock().unwrap().push(call);
            Ok(())
        })
    }
}

#[tokio::test(start_paused = true)]
async fn a_fake_editor_saves_the_finding_without_a_process() {
    let (tmp, store) = open().await;
    seed(&store, 1).await;
    let mut session = Session::load(&store).await.unwrap();
    let cache = tmp.path().join("cache");
    session.prepare(Some("ed".to_owned()), cache.clone(), false);
    let mode = Arc::new(Mutex::new(0));
    session.use_editor(Box::new(Rewrite {
        mode: Arc::clone(&mode),
    }));
    session.open_draft(&store).await.unwrap();
    session.edit_selected(&store).await.unwrap();
    let saved = store
        .call(|conn| {
            conn.query_row(
                "SELECT title || '|' || body FROM findings WHERE id = '1'",
                [],
                |row| row.get::<_, String>(0),
            )
        })
        .await
        .unwrap();
    assert_eq!(saved, "Kept|fresh body\n");
    assert_eq!(*mode.lock().unwrap(), 0o700);
    assert!(cache.read_dir().unwrap().next().is_none());
}

#[tokio::test]
async fn a_fake_browser_receives_only_the_pull_url() {
    let (_tmp, store) = open().await;
    seed(&store, 1).await;
    let mut session = Session::load(&store).await.unwrap();
    session.arm_actions(None, Some("viewer".to_owned()));
    let calls = Arc::new(Mutex::new(Vec::new()));
    session.use_browser(Box::new(Recorded {
        calls: Arc::clone(&calls),
    }));
    session.open_pull().await.unwrap();
    assert_eq!(session.notice(), None);
    assert_eq!(
        calls.lock().unwrap().as_slice(),
        [(
            "viewer".to_owned(),
            "https://github.com/o/r/pull/1".to_owned()
        )]
    );
    assert_eq!(queue::count(&store, "SELECT count(*) FROM jobs").await, 1);
}
