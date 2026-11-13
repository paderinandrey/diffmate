use super::seam::{AuthProbe, Probe, ProbeFuture};
use super::which::which;
use crate::app::AppConfig;
use crate::gh::GhCli;
use crate::proc::run_tool;

pub(super) struct Live<'a> {
    config: &'a AppConfig,
}

impl<'a> Live<'a> {
    pub(super) const fn new(config: &'a AppConfig) -> Self {
        Self { config }
    }
}

impl AuthProbe for Live<'_> {
    fn status(&self) -> ProbeFuture<'_> {
        Box::pin(async move {
            let path = self.config.env.get("PATH").unwrap_or_default();
            if which(path, "gh").await.is_none() {
                let shown = path.to_string_lossy();
                return Probe::Failed(format!("gh was not found in PATH: {shown}"));
            }
            let command = GhCli::new(&self.config.env).command(["auth", "status"]);
            match run_tool("gh", command).await {
                Ok(_) => Probe::Ready,
                Err(error) => Probe::Failed(format!("{error}; run gh auth login")),
            }
        })
    }
}

pub(super) async fn check(probe: &dyn AuthProbe) -> super::Check {
    match probe.status().await {
        Probe::Ready => super::Check::pass("gh auth: ok"),
        Probe::Unfinished => super::Check::fail("gh auth: probe did not finish"),
        Probe::Failed(detail) => super::Check::fail(format!("gh auth: {detail}")),
    }
}
