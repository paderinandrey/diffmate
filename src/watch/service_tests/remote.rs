use std::future::{Future, Ready, ready};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::watch;

use crate::agent::{Agent, AgentOutcome, AgentRequest};
use crate::error::Error;
use crate::gh::{
    Annotation, CheckRuns, CreatedReview, DirectRequest, Discussion, Gh, ListedReview, PrMeta,
    ReviewRequest,
};
use crate::pr_url::PrRef;
use crate::store::records::Metrics;

type Done<T> = Ready<Result<T, Error>>;
pub(super) struct Remote {
    pub posts: Arc<AtomicUsize>,
    pub hang: bool,
}
pub(super) struct Bot(pub Option<watch::Sender<bool>>);

fn gone<T>() -> Done<T> {
    ready(Err(Error::Command {
        program: "gh",
        args: "api".into(),
        detail: "unavailable".into(),
    }))
}
pub(super) fn head() -> String {
    "h".repeat(40)
}
fn meta(pr: &PrRef) -> PrMeta {
    PrMeta {
        number: pr.number,
        url: pr.web_url(),
        title: "change".into(),
        body: String::new(),
        author: "owner".into(),
        base_ref: "main".into(),
        head_ref: "branch".into(),
        head_sha: head(),
        is_cross_repository: false,
        state: "OPEN".into(),
        is_draft: false,
    }
}

#[allow(refining_impl_trait)]
impl Gh for Remote {
    fn pr_view(&self, pr: &PrRef) -> Done<PrMeta> {
        ready(Ok(meta(pr)))
    }
    fn discussion(&self, _: &PrRef) -> Done<Discussion> {
        gone()
    }
    fn check_runs(&self, _: &PrRef, _: &str) -> Done<CheckRuns> {
        gone()
    }
    fn annotations(&self, _: &PrRef, _: u64) -> Done<Vec<Annotation>> {
        gone()
    }
    fn search_review_requested(&self) -> Done<Vec<DirectRequest>> {
        ready(Ok(Vec::new()))
    }
    fn current_login(&self) -> Done<String> {
        ready(Ok("me".into()))
    }
    fn list_my_reviews(&self, _: &PrRef) -> Done<Vec<ListedReview>> {
        ready(Ok(Vec::new()))
    }
    fn create_review(
        &self,
        _: &PrRef,
        _: &ReviewRequest,
    ) -> impl Future<Output = Result<CreatedReview, Error>> + Send {
        self.posts.fetch_add(1, Ordering::Relaxed);
        let hang = self.hang;
        async move {
            if hang {
                std::future::pending::<()>().await;
            }
            Ok(CreatedReview { id: "9".into() })
        }
    }
}

impl Agent for Bot {
    async fn run(&self, request: &AgentRequest) -> AgentOutcome {
        let Some(gate) = &self.0 else {
            return finished();
        };
        gate.send_replace(true);
        let mut cancel = request.cancel_receiver().unwrap();
        crate::cancel::cancelled(&mut cancel).await;
        AgentOutcome::Cancelled {
            metrics: Metrics::default(),
        }
    }
}
fn finished() -> AgentOutcome {
    let raw = serde_json::json!({
        "status": "no_findings", "limitation": null,
        "findings": [], "future_improvements": []
    });
    AgentOutcome::Succeeded {
        output: serde_json::from_value(raw.clone()).unwrap(),
        raw,
        metrics: Metrics::default(),
    }
}
