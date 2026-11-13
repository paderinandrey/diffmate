use super::{miss, unavailable};
use crate::error::Error;
use crate::gh::{
    Annotation, CheckRuns, CommentLink, CreatedReview, DirectRequest, Discussion, Gh, PrMeta,
    ReviewRequest, ThreadState,
};
use crate::pr_url::PrRef;
use std::future::{Future, ready};
use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) struct Remote {
    comments: Option<Vec<CommentLink>>,
    threads: Option<Vec<ThreadState>>,
    posts: AtomicUsize,
}

impl Remote {
    pub(crate) fn comments_fail() -> Self {
        Self::new(None, None)
    }

    pub(crate) fn linked() -> Self {
        Self::new(Some(super::sample()), Some(vec![super::resolved()]))
    }

    pub(crate) fn status_fails() -> Self {
        Self::new(Some(super::sample()), None)
    }

    pub(crate) fn posts(&self) -> usize {
        self.posts.load(Ordering::SeqCst)
    }

    fn new(comments: Option<Vec<CommentLink>>, threads: Option<Vec<ThreadState>>) -> Self {
        Self {
            comments,
            threads,
            posts: AtomicUsize::new(0),
        }
    }
}

impl Gh for Remote {
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
        _: &ReviewRequest,
    ) -> impl Future<Output = Result<CreatedReview, Error>> + Send {
        self.posts.fetch_add(1, Ordering::SeqCst);
        miss()
    }
    fn review_comment_threads(
        &self,
        _: &PrRef,
        _: &str,
    ) -> impl Future<Output = Result<Vec<CommentLink>, Error>> + Send {
        ready(self.comments.clone().ok_or_else(unavailable))
    }
    fn thread_resolved(
        &self,
        _: &PrRef,
    ) -> impl Future<Output = Result<Vec<ThreadState>, Error>> + Send {
        ready(self.threads.clone().ok_or_else(unavailable))
    }
}
