use super::queue::HEAD;
use crate::error::Error;
use crate::gh::{
    Annotation, CheckRuns, CreatedReview, DirectRequest, Discussion, Gh, PrMeta, ReviewRequest,
};
use crate::pr_url::PrRef;
use std::future::Future;

pub(super) struct Hang;

fn miss<T>() -> std::future::Ready<Result<T, Error>> {
    std::future::ready(Err(Error::Command {
        program: "gh",
        args: String::new(),
        detail: "unused".into(),
    }))
}

impl Gh for Hang {
    fn pr_view(&self, pr: &PrRef) -> impl Future<Output = Result<PrMeta, Error>> + Send {
        let number = pr.number;
        async move {
            Ok(PrMeta {
                number,
                url: String::new(),
                title: String::new(),
                body: String::new(),
                author: String::new(),
                base_ref: String::new(),
                head_ref: String::new(),
                head_sha: HEAD.to_string(),
                is_cross_repository: false,
                state: String::new(),
                is_draft: false,
            })
        }
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
        std::future::ready(Ok("me".to_string()))
    }
    fn list_my_reviews(
        &self,
        _: &PrRef,
    ) -> impl Future<Output = Result<Vec<crate::gh::ListedReview>, Error>> + Send {
        std::future::ready(Ok(Vec::new()))
    }
    fn create_review(
        &self,
        _: &PrRef,
        _: &ReviewRequest,
    ) -> impl Future<Output = Result<CreatedReview, Error>> + Send {
        std::future::pending()
    }
}
