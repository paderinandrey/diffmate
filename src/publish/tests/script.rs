use super::support::cell;
use crate::error::Error;
use crate::gh::{
    Annotation, CheckRuns, CreatedReview, DirectRequest, Discussion, Gh, PrMeta, ReviewRequest,
};
use crate::pr_url::PrRef;
use crate::store::Store;
use std::future::{Future, ready};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

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
    posts: AtomicUsize,
}

impl Script {
    pub(super) fn new(store: Store) -> Self {
        Self {
            store,
            seen: Mutex::new(Vec::new()),
            posts: AtomicUsize::new(0),
        }
    }

    pub(super) fn seen(&self) -> Vec<ReviewRequest> {
        self.seen.lock().unwrap().clone()
    }

    pub(super) fn posts(&self) -> usize {
        self.posts.load(Ordering::SeqCst)
    }
}

impl Gh for Script {
    fn pr_view(&self, pr: &PrRef) -> impl Future<Output = Result<PrMeta, Error>> + Send {
        let store = self.store.clone();
        let pr = pr.clone();
        async move {
            let head_sha = confirmed(&store, &pr).await?;
            Ok(PrMeta {
                number: pr.number,
                url: String::new(),
                title: String::new(),
                body: String::new(),
                author: String::new(),
                base_ref: String::new(),
                head_ref: String::new(),
                head_sha,
                is_cross_repository: false,
                state: String::new(),
                is_draft: false,
            })
        }
    }
    fn current_login(&self) -> impl Future<Output = Result<String, Error>> + Send {
        ready(Ok("tester".into()))
    }
    fn list_my_reviews(
        &self,
        _: &PrRef,
    ) -> impl Future<Output = Result<Vec<crate::gh::ListedReview>, Error>> + Send {
        ready(Ok(Vec::new()))
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
            self.posts.fetch_add(1, Ordering::SeqCst);
            let state = cell::<String>(&self.store, "SELECT state FROM publications").await;
            assert_eq!(state, "sending", "{state}");
            self.seen.lock().unwrap().push(review);
            Ok(CreatedReview { id: "9001".into() })
        }
    }
}

async fn confirmed(store: &crate::store::Store, pr: &PrRef) -> Result<String, Error> {
    let owner = pr.owner.clone();
    let repo = pr.repo.clone();
    let number = i64::try_from(pr.number).unwrap();
    store
        .call(move |conn| {
            let sql = "SELECT p.confirmed_sha FROM publications p \
                       JOIN targets t ON t.id = p.target_id \
                       WHERE t.owner = ?1 AND t.repo = ?2 AND t.number = ?3 \
                       ORDER BY p.rowid DESC LIMIT 1";
            conn.query_row(sql, rusqlite::params![owner, repo, number], |row| {
                row.get(0)
            })
        })
        .await
}

struct Plain;

impl Gh for Plain {
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
}

#[tokio::test]
async fn defaults_refuse_publication() {
    let plain = Plain;
    let pr = PrRef {
        owner: "o".into(),
        repo: "r".into(),
        number: 1,
    };
    let login = Gh::current_login(&plain).await.unwrap_err();
    let listed = Gh::list_my_reviews(&plain, &pr).await.unwrap_err();
    assert!(login.to_string().contains("publication is not available"));
    assert!(listed.to_string().contains("publication is not available"));
}
