use super::fixture::{self, mode};
use super::publish::{self, paths};
use super::queue::{count, key, open};
use crate::config::Settings;
use crate::error::Error;
use crate::gh::{Annotation, CheckRuns, DirectRequest, Discussion, Gh, PrMeta};
use crate::pr_url::PrRef;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::future::Future;

struct StallView;

fn pending<T>() -> impl Future<Output = Result<T, Error>> + Send {
    std::future::pending()
}

impl Gh for StallView {
    fn pr_view(&self, _: &PrRef) -> impl Future<Output = Result<PrMeta, Error>> + Send {
        pending()
    }
    fn discussion(&self, _: &PrRef) -> impl Future<Output = Result<Discussion, Error>> + Send {
        pending()
    }
    fn check_runs(
        &self,
        _: &PrRef,
        _: &str,
    ) -> impl Future<Output = Result<CheckRuns, Error>> + Send {
        pending()
    }
    fn annotations(
        &self,
        _: &PrRef,
        _: u64,
    ) -> impl Future<Output = Result<Vec<Annotation>, Error>> + Send {
        pending()
    }
    fn search_review_requested(
        &self,
    ) -> impl Future<Output = Result<Vec<DirectRequest>, Error>> + Send {
        pending()
    }
}

async fn quit_during_head_check(pressed: KeyEvent) {
    let (tmp, store) = open().await;
    publish::seed(&store).await;
    let paths = paths(&tmp);
    let mut session = fixture::open_session(&store, &paths.lock()).await;
    let gh = StallView;
    let git = crate::git::GitCli::new(&crate::procenv::InheritedEnv::from_pairs(Vec::new()));
    let settings = Settings::default();
    let mode = mode(&gh, &git, &paths, &settings);
    super::super::apply(&mut session, &store, &mode, key(KeyCode::Enter), None)
        .await
        .unwrap();
    let gate = std::sync::Arc::new(super::super::reader::KeyGate::new());
    let (tx, mut keys) = tokio::sync::mpsc::channel(4);
    let quit = {
        let mut turn = std::pin::pin!(super::super::apply(
            &mut session,
            &store,
            &mode,
            key(KeyCode::Char('c')),
            Some((&gate, &mut keys)),
        ));
        tx.send(Ok(Some(pressed))).await.unwrap();
        turn.as_mut().await.unwrap()
    };
    assert!(quit);
    assert_eq!(count(&store, "SELECT count(*) FROM publications").await, 0);
    assert_eq!(session.notice(), None);
}

#[tokio::test]
async fn quitting_during_the_head_check_publishes_nothing() {
    quit_during_head_check(key(KeyCode::Char('q'))).await;
    quit_during_head_check(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)).await;
}
