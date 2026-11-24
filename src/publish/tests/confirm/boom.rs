use super::RUN;
use crate::error::Error;
use crate::gh::{Annotation, CheckRuns, DirectRequest, Discussion, Gh, PrMeta};
use crate::pr_url::PrRef;
use std::future::{Future, ready};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Check {
    Head,
    Login,
    Reviews,
}

pub(super) struct Boom {
    detail: Option<&'static str>,
    check: Check,
    stalls: usize,
    logins: AtomicUsize,
    posts: AtomicUsize,
    views: AtomicUsize,
    lists: AtomicUsize,
}

impl Boom {
    pub(super) fn new(detail: &'static str, view: bool) -> Self {
        Self {
            detail: Some(detail),
            check: if view { Check::Head } else { Check::Reviews },
            stalls: 0,
            logins: AtomicUsize::new(0),
            posts: AtomicUsize::new(0),
            views: AtomicUsize::new(0),
            lists: AtomicUsize::new(0),
        }
    }

    pub(super) fn stalling(check: Check, stalls: usize) -> Self {
        Self {
            check,
            stalls,
            detail: None,
            ..Self::new("", false)
        }
    }

    async fn check(&self, check: Check, calls: &AtomicUsize) -> Result<(), Error> {
        let previous = calls.fetch_add(1, Ordering::Relaxed);
        if self.check == check {
            if previous < self.stalls {
                return std::future::pending().await;
            }
            if let Some(detail) = self.detail {
                return Err(cmd(detail));
            }
        }
        Ok(())
    }

    pub(super) fn calls(&self) -> [usize; 4] {
        [
            self.views(),
            self.logins.load(Ordering::Relaxed),
            self.lists(),
            self.posts.load(Ordering::Relaxed),
        ]
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
    async fn pr_view(&self, pr: &PrRef) -> Result<PrMeta, Error> {
        self.check(Check::Head, &self.views).await?;
        Ok(meta(pr.number))
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
    async fn current_login(&self) -> Result<String, Error> {
        self.check(Check::Login, &self.logins).await?;
        Ok("me".into())
    }
    async fn list_my_reviews(&self, _: &PrRef) -> Result<Vec<crate::gh::ListedReview>, Error> {
        self.check(Check::Reviews, &self.lists).await?;
        Ok(Vec::new())
    }
    fn create_review(
        &self,
        _: &PrRef,
        _: &crate::gh::ReviewRequest,
    ) -> impl Future<Output = Result<crate::gh::CreatedReview, Error>> {
        let _ = self.posts.fetch_add(1, Ordering::Relaxed);
        ready(Ok(crate::gh::CreatedReview { id: "7".into() }))
    }
}
