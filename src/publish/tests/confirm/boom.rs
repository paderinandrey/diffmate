use super::RUN;
use crate::error::Error;
use crate::gh::{Annotation, CheckRuns, DirectRequest, Discussion, Gh, PrMeta};
use crate::pr_url::PrRef;
use std::future::{Future, ready};
use std::sync::atomic::{AtomicUsize, Ordering};

pub(super) struct Boom {
    detail: &'static str,
    view: bool,
    views: AtomicUsize,
    lists: AtomicUsize,
}

impl Boom {
    pub(super) fn new(detail: &'static str, view: bool) -> Self {
        Self {
            detail,
            view,
            views: AtomicUsize::new(0),
            lists: AtomicUsize::new(0),
        }
    }

    pub(super) fn views(&self) -> usize {
        self.views.load(Ordering::Relaxed)
    }

    pub(super) fn lists(&self) -> usize {
        self.lists.load(Ordering::Relaxed)
    }
}

fn cmd(detail: &str) -> Error {
    Error::Command {
        program: "gh",
        args: String::new(),
        detail: detail.into(),
    }
}

#[rustfmt::skip]
fn meta(number: u64) -> PrMeta {
    PrMeta { number, url: String::new(), title: String::new(), body: String::new(), author: String::new(), base_ref: String::new(), head_ref: String::new(), head_sha: RUN.into(), is_cross_repository: false, state: String::new(), is_draft: false }
}

fn miss<T>() -> std::future::Ready<Result<T, Error>> {
    ready(Err(cmd("unused")))
}

impl Gh for Boom {
    fn pr_view(&self, pr: &PrRef) -> impl Future<Output = Result<PrMeta, Error>> + Send {
        let _hit = self.views.fetch_add(1, Ordering::Relaxed);
        ready(if self.view {
            Err(cmd(self.detail))
        } else {
            Ok(meta(pr.number))
        })
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
        ready(Ok("me".into()))
    }
    fn list_my_reviews(
        &self,
        _: &PrRef,
    ) -> impl Future<Output = Result<Vec<crate::gh::ListedReview>, Error>> + Send {
        let _hit = self.lists.fetch_add(1, Ordering::Relaxed);
        ready(if self.view {
            Ok(Vec::new())
        } else {
            Err(cmd(self.detail))
        })
    }
}
