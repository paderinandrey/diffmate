use crate::error::Error;
use crate::gh::{
    Annotation, CheckRuns, CreatedReview, DirectRequest, Discussion, Gh, PrMeta, ReviewRequest,
};
use crate::pr_url::PrRef;
use std::sync::Mutex;

pub(crate) struct Fake {
    heads: Mutex<Vec<String>>,
    reviews: Mutex<Vec<crate::gh::ListedReview>>,
    seen: Mutex<Vec<ReviewRequest>>,
}

impl Fake {
    pub(crate) fn new(head: &str) -> Self {
        Self {
            heads: Mutex::new(vec![head.to_string()]),
            reviews: Mutex::new(Vec::new()),
            seen: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn then(&self, head: &str) {
        self.heads.lock().unwrap().push(head.to_string());
    }

    pub(crate) fn set_reviews(&self, reviews: Vec<crate::gh::ListedReview>) {
        *self.reviews.lock().unwrap() = reviews;
    }

    pub(crate) fn reviews(&self) -> Vec<crate::gh::ListedReview> {
        self.reviews.lock().unwrap().clone()
    }

    pub(crate) fn seen(&self) -> Vec<ReviewRequest> {
        self.seen.lock().unwrap().clone()
    }

    fn next_head(&self) -> String {
        let mut heads = self.heads.lock().unwrap();
        if heads.len() > 1 {
            heads.remove(0)
        } else {
            heads[0].clone()
        }
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
            head_sha: self.next_head(),
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
    fn current_login(&self) -> std::future::Ready<Result<String, Error>> {
        std::future::ready(Ok("me".into()))
    }
    fn list_my_reviews(
        &self,
        _: &PrRef,
    ) -> std::future::Ready<Result<Vec<crate::gh::ListedReview>, Error>> {
        std::future::ready(Ok(self.reviews()))
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
