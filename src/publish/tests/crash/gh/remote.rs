use std::future::ready;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::error::Error;
use crate::gh::{ListedReview, PrMeta};
use crate::pr_url::PrRef;
use crate::store::Store;

#[derive(Clone, Copy)]
pub(super) enum Mode {
    Post,
    Wait,
    Code(u16),
}

pub(in crate::publish::tests::crash) struct Remote {
    pub(super) store: Store,
    reviews: Mutex<Vec<ListedReview>>,
    pub(super) posts: AtomicUsize,
    pub(super) mode: Mutex<Mode>,
}

impl Remote {
    pub(in crate::publish::tests::crash) fn new(store: Store) -> Self {
        Self {
            store,
            reviews: Mutex::new(Vec::new()),
            posts: AtomicUsize::new(0),
            mode: Mutex::new(Mode::Post),
        }
    }

    pub(in crate::publish::tests::crash) fn hang(&self) {
        *self.mode.lock().unwrap() = Mode::Wait;
    }

    pub(in crate::publish::tests::crash) fn fail(&self, code: u16) {
        *self.mode.lock().unwrap() = Mode::Code(code);
    }

    pub(in crate::publish::tests::crash) fn reviews(&self, reviews: Vec<ListedReview>) {
        *self.reviews.lock().unwrap() = reviews;
    }

    pub(in crate::publish::tests::crash) fn posts(&self) -> usize {
        self.posts.load(Ordering::SeqCst)
    }

    pub(super) fn listed(&self) -> Vec<ListedReview> {
        self.reviews.lock().unwrap().clone()
    }
}

pub(super) fn miss<T>() -> std::future::Ready<Result<T, Error>> {
    ready(Err(Error::Command {
        program: "gh",
        args: String::new(),
        detail: "unused".into(),
    }))
}

pub(super) fn meta(pr: &PrRef) -> PrMeta {
    PrMeta {
        number: pr.number,
        url: String::new(),
        title: String::new(),
        body: String::new(),
        author: String::new(),
        base_ref: String::new(),
        head_ref: String::new(),
        head_sha: "abc123".into(),
        is_cross_repository: false,
        state: String::new(),
        is_draft: false,
    }
}
