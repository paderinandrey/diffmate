use std::sync::atomic::{AtomicUsize, Ordering};

use super::seam::{AgentProbe, AuthProbe, Probe, ProbeFuture};

struct Auth(Probe);

impl AuthProbe for Auth {
    fn status(&self) -> ProbeFuture<'_> {
        let outcome = self.0.clone();
        Box::pin(async move { outcome })
    }
}

struct Agent {
    version: Probe,
    help: Probe,
    help_calls: AtomicUsize,
}

impl AgentProbe for Agent {
    fn version(&self) -> ProbeFuture<'_> {
        let outcome = self.version.clone();
        Box::pin(async move { outcome })
    }

    fn help(&self) -> ProbeFuture<'_> {
        self.help_calls.fetch_add(1, Ordering::SeqCst);
        let outcome = self.help.clone();
        Box::pin(async move { outcome })
    }
}

fn agent(version: Probe, help: Probe) -> Agent {
    Agent {
        version,
        help,
        help_calls: AtomicUsize::new(0),
    }
}

#[tokio::test]
async fn auth_failure_names_login() {
    let failed = Probe::Failed("not logged in".to_string());
    let check = super::gh_auth::check(&Auth(failed)).await;
    assert!(!check.passed);
    assert_eq!(check.text, "gh auth: not logged in");
}

#[tokio::test]
async fn auth_success_is_ok() {
    let check = super::gh_auth::check(&Auth(Probe::Ready)).await;
    assert!(check.passed);
    assert_eq!(check.text, "gh auth: ok");
}

#[tokio::test]
async fn old_grok_skips_the_help_probe() {
    let agent = agent(
        Probe::Failed("older than the minimum".to_string()),
        Probe::Ready,
    );
    let check = super::grok::check(&agent).await;
    assert!(!check.passed);
    assert!(
        check.text.contains("older than the minimum"),
        "{}",
        check.text
    );
    assert_eq!(agent.help_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn grok_help_failure_is_reported() {
    let agent = agent(
        Probe::Ready,
        Probe::Failed("does not support --sandbox".to_string()),
    );
    let check = super::grok::check(&agent).await;
    assert!(!check.passed);
    assert!(
        check.text.contains("does not support --sandbox"),
        "{}",
        check.text
    );
    assert_eq!(agent.help_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn grok_success_is_ok() {
    let check = super::grok::check(&agent(Probe::Ready, Probe::Ready)).await;
    assert!(check.passed);
    assert_eq!(check.text, "grok: ok");
}
