mod git;
pub(crate) use git::FakeGit;

use crate::agent::{Agent, AgentOutcome, AgentRequest};
use crate::error::Error;
use crate::gh::{Annotation, CheckRuns, DirectRequest, Discussion, Gh, PrMeta};
use crate::pr_url::PrRef;
use crate::store::records::Metrics;
use std::future::{Ready, ready};
use std::time::Duration;

type Done<T> = Ready<Result<T, Error>>;
pub(super) struct FakeGh;
pub(super) enum FakeAgent {
    WaitForCancel,
    FinishAfter(Duration),
}

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
            is_draft: false,
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
    fn search_review_requested(&self) -> Done<Vec<DirectRequest>> {
        ready(Ok(Vec::new()))
    }
}

impl Agent for FakeAgent {
    async fn run(&self, request: &AgentRequest) -> AgentOutcome {
        let mut cancel = request.cancel_receiver().unwrap();
        if let Self::FinishAfter(delay) = self
            && crate::cancel::raced(&mut cancel, tokio::time::sleep(*delay))
                .await
                .is_some()
        {
            let raw = serde_json::json!({
                "status": "no_findings", "limitation": null,
                "findings": [], "future_improvements": []
            });
            return AgentOutcome::Succeeded {
                output: serde_json::from_value(raw.clone()).unwrap(),
                raw,
                metrics: Metrics::default(),
            };
        }
        crate::cancel::cancelled(&mut cancel).await;
        AgentOutcome::Cancelled {
            metrics: Metrics::default(),
        }
    }
}
