use std::future::{Ready, ready};
type Done<T> = Ready<Result<T, Error>>;

use crate::error::Error;
use crate::gh::{Annotation, CheckRuns, DirectRequest, Discussion, Gh, PrMeta};
use crate::pr_url::PrRef;

pub(super) struct FakeGh;

#[allow(refining_impl_trait)]
impl Gh for FakeGh {
    fn pr_view(&self, pr: &PrRef) -> Done<PrMeta> {
        ready(Ok(PrMeta {
            number: pr.number,
            url: pr.web_url(),
            title: "changed".into(),
            body: String::new(),
            author: "owner".into(),
            base_ref: "new-main".into(),
            head_ref: "branch".into(),
            head_sha: "new-head".into(),
            is_cross_repository: false,
            state: "OPEN".into(),
            is_draft: false,
        }))
    }
    fn discussion(&self, _: &PrRef) -> Done<Discussion> {
        unreachable!()
    }
    fn check_runs(&self, _: &PrRef, _: &str) -> Done<CheckRuns> {
        unreachable!()
    }
    fn annotations(&self, _: &PrRef, _: u64) -> Done<Vec<Annotation>> {
        unreachable!()
    }
    fn search_review_requested(&self) -> Done<Vec<DirectRequest>> {
        ready(Ok(Vec::new()))
    }
}
