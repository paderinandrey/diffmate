use super::support::cell;
use crate::error::Error;
use crate::gh::{
    Annotation, CheckRuns, CreatedReview, DirectRequest, Discussion, Gh, PrMeta, ReviewRequest,
};
use crate::pr_url::PrRef;
use crate::store::Store;
use std::future::{Future, ready};
use std::sync::Mutex;

fn miss<T>() -> std::future::Ready<Result<T, Error>> {
    ready(Err(Error::Command {
        program: "gh",
        args: String::new(),
        detail: "unused".into(),
    }))
}

pub(super) struct Script {
    store: Store,
    seen: Mutex<Vec<ReviewRequest>>,
}

impl Script {
    pub(super) fn new(store: Store) -> Self {
        Self {
            store,
            seen: Mutex::new(Vec::new()),
        }
    }

    pub(super) fn seen(&self) -> Vec<ReviewRequest> {
        self.seen.lock().unwrap().clone()
    }
}

impl Gh for Script {
    fn pr_view(&self, _: &PrRef) -> impl Future<Output = Result<PrMeta, Error>> + Send {
        miss()
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
    fn create_review(
        &self,
        _: &PrRef,
        review: &ReviewRequest,
    ) -> impl Future<Output = Result<CreatedReview, Error>> + Send {
        let review = review.clone();
        async move {
            let state = cell::<String>(&self.store, "SELECT state FROM publications").await;
            assert_eq!(state, "sending", "{state}");
            self.seen.lock().unwrap().push(review);
            Ok(CreatedReview { id: "9001".into() })
        }
    }
}

#[tokio::test]
async fn defaults_refuse_publication() {
    let (_tmp, store) = super::support::open().await;
    let script = Script::new(store);
    let pr = PrRef {
        owner: "o".into(),
        repo: "r".into(),
        number: 1,
    };
    let login = Gh::current_login(&script).await.unwrap_err();
    let listed = Gh::list_my_reviews(&script, &pr).await.unwrap_err();
    assert!(login.to_string().contains("publication is not available"));
    assert!(listed.to_string().contains("publication is not available"));
}
