use crate::error::Error;
use crate::gh::{
    Annotation, CheckRuns, CreatedReview, DirectRequest, Discussion, Gh, PrMeta, ReviewRequest,
};
use crate::pr_url::PrRef;
use std::sync::Mutex;

pub(crate) struct Fake {
    head: String,
    seen: Mutex<Vec<ReviewRequest>>,
}

impl Fake {
    pub(crate) fn new(head: &str) -> Self {
        Self {
            head: head.to_string(),
            seen: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn seen(&self) -> Vec<ReviewRequest> {
        self.seen.lock().unwrap().clone()
    }
}

fn unused<T>() -> std::future::Ready<Result<T, Error>> {
    std::future::ready(Err(Error::Command {
        program: "gh",
        args: String::new(),
        detail: "unused".into(),
    }))
}

#[allow(refining_impl_trait)]
impl Gh for Fake {
    fn pr_view(&self, pr: &PrRef) -> std::future::Ready<Result<PrMeta, Error>> {
        std::future::ready(Ok(PrMeta {
            number: pr.number,
            url: String::new(),
            title: String::new(),
            body: String::new(),
            author: String::new(),
            base_ref: String::new(),
            head_ref: String::new(),
            head_sha: self.head.clone(),
            is_cross_repository: false,
            state: String::new(),
            is_draft: false,
        }))
    }
    fn discussion(&self, _: &PrRef) -> std::future::Ready<Result<Discussion, Error>> {
        unused()
    }
    fn check_runs(&self, _: &PrRef, _: &str) -> std::future::Ready<Result<CheckRuns, Error>> {
        unused()
    }
    fn annotations(&self, _: &PrRef, _: u64) -> std::future::Ready<Result<Vec<Annotation>, Error>> {
        unused()
    }
    fn search_review_requested(&self) -> std::future::Ready<Result<Vec<DirectRequest>, Error>> {
        unused()
    }
    fn create_review(
        &self,
        _: &PrRef,
        review: &ReviewRequest,
    ) -> impl std::future::Future<Output = Result<CreatedReview, Error>> + Send {
        let review = review.clone();
        async move {
            self.seen.lock().unwrap().push(review);
            Ok(CreatedReview { id: "9001".into() })
        }
    }
}
