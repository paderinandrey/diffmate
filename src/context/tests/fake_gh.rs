use crate::error::Error;
use crate::gh::{Annotation, CheckRuns, DirectRequest, Discussion, Gh, PrMeta, ThreadState};
use crate::pr_url::PrRef;
use std::future::ready;
use std::sync::Mutex;

type Done<T> = std::future::Ready<Result<T, Error>>;

#[derive(Default)]
pub(super) struct FakeGh {
    pub discussion: Mutex<Option<Discussion>>,
    pub runs: Mutex<Option<CheckRuns>>,
    pub annotations_fail: bool,
    pub annotated: Mutex<Vec<u64>>,
    pub threads: Mutex<Option<Vec<ThreadState>>>,
}

fn broken<T>() -> Done<T> {
    ready(Err(Error::Command {
        program: "gh",
        args: "api".into(),
        detail: "boom\u{1b}[31m\nsecond line".into(),
    }))
}

#[allow(refining_impl_trait)]
impl Gh for FakeGh {
    fn pr_view(&self, _: &PrRef) -> Done<PrMeta> {
        broken()
    }

    fn discussion(&self, _: &PrRef) -> Done<Discussion> {
        self.discussion
            .lock()
            .unwrap()
            .take()
            .map_or_else(broken, |found| ready(Ok(found)))
    }

    fn check_runs(&self, _: &PrRef, _: &str) -> Done<CheckRuns> {
        self.runs
            .lock()
            .unwrap()
            .take()
            .map_or_else(broken, |found| ready(Ok(found)))
    }

    fn search_review_requested(&self) -> Done<Vec<DirectRequest>> {
        ready(Ok(Vec::new()))
    }

    fn annotations(&self, _: &PrRef, id: u64) -> Done<Vec<Annotation>> {
        self.annotated.lock().unwrap().push(id);
        if self.annotations_fail {
            return broken();
        }
        let message = Some("boom".to_string());
        let found = Annotation {
            path: "src/a.rs".into(),
            start_line: 7,
            level: Some("failure".into()),
            message,
        };
        ready(Ok(vec![found]))
    }

    fn thread_resolved(&self, _: &PrRef) -> Done<Vec<ThreadState>> {
        self.threads.lock().unwrap().clone().map_or_else(
            || {
                ready(Err(Error::Command {
                    program: "gh",
                    args: String::new(),
                    detail: "publication is not available on this gh".into(),
                }))
            },
            |rows| ready(Ok(rows)),
        )
    }
}

pub(super) fn pr() -> PrRef {
    PrRef {
        owner: "o".into(),
        repo: "r".into(),
        number: 1,
    }
}
