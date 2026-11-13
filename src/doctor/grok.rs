use super::seam::{AgentProbe, Probe, ProbeFuture};
use super::which::which;
use crate::agent::claude::AUTH_ENV;
use crate::agent::probe::{check_flags, check_version};
use crate::app::AppConfig;
use std::ffi::OsStr;

pub(super) struct Live<'a> {
    config: &'a AppConfig,
}

impl<'a> Live<'a> {
    pub(super) const fn new(config: &'a AppConfig) -> Self {
        Self { config }
    }
}

impl AgentProbe for Live<'_> {
    fn version(&self) -> ProbeFuture<'_> {
        Box::pin(async move { version(self.config).await })
    }

    fn help(&self) -> ProbeFuture<'_> {
        Box::pin(async move { help(self.config).await })
    }
}

async fn version(config: &AppConfig) -> Probe {
    let path = config.env.get("PATH").unwrap_or_default();
    if which(path, "grok").await.is_none() {
        let shown = path.to_string_lossy();
        return Probe::Failed(format!("grok was not found in PATH: {shown}"));
    }
    let env = config.env.pick(&AUTH_ENV);
    match check_version(OsStr::new("grok"), &env, None).await {
        Ok(Some(())) => Probe::Ready,
        Ok(None) => Probe::Unfinished,
        Err(detail) => Probe::Failed(detail.message),
    }
}

async fn help(config: &AppConfig) -> Probe {
    let env = config.env.pick(&AUTH_ENV);
    match check_flags(OsStr::new("grok"), &env, None).await {
        Ok(Some(())) => Probe::Ready,
        Ok(None) => Probe::Unfinished,
        Err(detail) => Probe::Failed(detail.message),
    }
}

pub(super) async fn check(probe: &dyn AgentProbe) -> super::Check {
    match probe.version().await {
        Probe::Ready => {}
        Probe::Unfinished => return super::Check::fail("grok: probe did not finish"),
        Probe::Failed(detail) => return super::Check::fail(format!("grok: {detail}")),
    }
    match probe.help().await {
        Probe::Ready => super::Check::pass("grok: ok"),
        Probe::Unfinished => super::Check::fail("grok: probe did not finish"),
        Probe::Failed(detail) => super::Check::fail(format!("grok: {detail}")),
    }
}
