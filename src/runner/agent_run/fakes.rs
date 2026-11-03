mod git;
pub(super) use git::FakeGit;

use crate::agent::{Agent, AgentOutcome, AgentRequest};
use crate::error::Error;
use crate::gh::{Annotation, CheckRuns, Discussion, Gh, PrMeta};
use crate::pr_url::PrRef;
use crate::store::records::Metrics;
use std::future::{Ready, ready};

type Done<T> = Ready<Result<T, Error>>;
pub(super) struct FakeGh;
pub(super) struct FakeAgent;

fn unavailable<T>() -> Done<T> {
    ready(Err(Error::Command {
        program: "gh",
        args: "api".into(),
        detail: "unavailable in fixture".into(),
    }))
}

#[allow(refining_impl_trait)]
impl Gh for FakeGh {
    fn pr_view(&self, pr: &PrRef) -> Done<PrMeta> {
        ready(Ok(PrMeta {
            number: pr.number,
            url: pr.web_url(),
            title: "change".into(),
            body: String::new(),
            author: "owner".into(),
            base_ref: "main".into(),
            head_ref: "branch".into(),
            head_sha: "h".repeat(40),
            is_cross_repository: false,
            state: "OPEN".into(),
        }))
    }
    fn discussion(&self, _: &PrRef) -> Done<Discussion> {
        unavailable()
    }
    fn check_runs(&self, _: &PrRef, _: &str) -> Done<CheckRuns> {
        unavailable()
    }
    fn annotations(&self, _: &PrRef, _: u64) -> Done<Vec<Annotation>> {
        unavailable()
    }
}

impl Agent for FakeAgent {
    async fn run(&self, request: &AgentRequest) -> AgentOutcome {
        let mut cancel = request.cancel_receiver().unwrap();
        crate::cancel::cancelled(&mut cancel).await;
        AgentOutcome::Cancelled {
            metrics: Metrics::default(),
        }
    }
}
