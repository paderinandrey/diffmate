mod checks;
mod discussion;
mod meta;

use crate::error::Error;
use crate::pr_url::PrRef;
use crate::proc::run_tool;
use crate::procenv::{InheritedEnv, Pairs};
use std::ffi::OsStr;
use std::future::Future;
use tokio::process::Command;

pub(crate) use checks::{Annotation, CheckRun, CheckRuns};
pub(crate) use discussion::{Comment, Discussion, Review, Thread};
pub(crate) use meta::PrMeta;

pub(crate) trait Gh {
    fn pr_view(&self, pr: &PrRef) -> impl Future<Output = Result<PrMeta, Error>> + Send;
    fn discussion(&self, pr: &PrRef) -> impl Future<Output = Result<Discussion, Error>> + Send;
    fn check_runs(
        &self,
        pr: &PrRef,
        sha: &str,
    ) -> impl Future<Output = Result<CheckRuns, Error>> + Send;
    fn annotations(
        &self,
        pr: &PrRef,
        check_run_id: u64,
    ) -> impl Future<Output = Result<Vec<Annotation>, Error>> + Send;
}

pub(crate) struct GhCli {
    env: Pairs,
}

impl GhCli {
    pub fn new(env: &InheritedEnv) -> GhCli {
        let mut vars = env.tool_env();
        for (name, value) in [
            ("GH_PROMPT_DISABLED", "1"),
            ("GH_NO_UPDATE_NOTIFIER", "1"),
            ("NO_COLOR", "1"),
        ] {
            vars.push((name.into(), value.into()));
        }
        GhCli { env: vars }
    }

    fn command(&self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Command {
        let mut command = Command::new("gh");
        command.args(args).env_clear();
        command.envs(self.env.iter().map(|(name, value)| (name, value)));
        command
    }
}

impl Gh for GhCli {
    async fn pr_view(&self, pr: &PrRef) -> Result<PrMeta, Error> {
        let url = pr.web_url();
        let args = ["pr", "view", url.as_str(), "--json", meta::FIELDS];
        let stdout = Box::pin(run_tool("gh", self.command(args))).await?;
        meta::parse(&stdout)
    }

    async fn discussion(&self, pr: &PrRef) -> Result<Discussion, Error> {
        discussion::fetch(self, pr).await
    }

    async fn check_runs(&self, pr: &PrRef, sha: &str) -> Result<CheckRuns, Error> {
        checks::runs(self, pr, sha).await
    }

    async fn annotations(&self, pr: &PrRef, check_run_id: u64) -> Result<Vec<Annotation>, Error> {
        checks::annotations(self, pr, check_run_id).await
    }
}
