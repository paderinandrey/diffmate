mod remote;

use std::future::Future;

pub(in crate::publish::tests::crash) use remote::Remote;
use remote::{Mode, meta, miss};

use crate::error::Error;
use crate::gh::{
    Annotation, CheckRuns, CreatedReview, DirectRequest, Discussion, Gh, ListedReview, PrMeta,
    ReviewRequest,
};
use crate::pr_url::PrRef;

impl Gh for Remote {
    fn pr_view(&self, pr: &PrRef) -> impl Future<Output = Result<PrMeta, Error>> + Send {
        std::future::ready(Ok(meta(pr)))
    }
    fn discussion(&self, _: &PrRef) -> impl Future<Output = Result<Discussion, Error>> + Send {
        miss()
    }
    fn check_runs(
        &self,
        _: &PrRef,
        _: &str,
    ) -> impl Future<Output = Result<CheckRuns, Error>> + Send {
        miss()
    }
    fn annotations(
        &self,
        _: &PrRef,
        _: u64,
    ) -> impl Future<Output = Result<Vec<Annotation>, Error>> + Send {
        miss()
    }
    fn search_review_requested(
        &self,
    ) -> impl Future<Output = Result<Vec<DirectRequest>, Error>> + Send {
        miss()
    }
    fn current_login(&self) -> impl Future<Output = Result<String, Error>> + Send {
        std::future::ready(Ok("tester".into()))
    }
    fn list_my_reviews(
        &self,
        _: &PrRef,
    ) -> impl Future<Output = Result<Vec<ListedReview>, Error>> + Send {
        std::future::ready(Ok(self.listed()))
    }
    fn create_review(
        &self,
        _: &PrRef,
        _: &ReviewRequest,
    ) -> impl Future<Output = Result<CreatedReview, Error>> + Send {
        self.posts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mode = *self.mode.lock().unwrap();
        let store = self.store.clone();
        async move {
            let sql = "SELECT state FROM publications ORDER BY state = 'sending' DESC LIMIT 1";
            let state = super::super::support::cell::<String>(&store, sql).await;
            assert_eq!(state, "sending", "{state}");
            match mode {
                Mode::Post => Ok(CreatedReview { id: "9001".into() }),
                Mode::Wait => std::future::pending().await,
                Mode::Code(code) => Err(Error::Command {
                    program: "gh",
                    args: String::new(),
                    detail: format!("HTTP {code}"),
                }),
            }
        }
    }
}
