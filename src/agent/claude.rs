mod contract;

use super::envelope::classify;
use super::schema::Schemas;
use super::{Agent, AgentOutcome, AgentRequest, FailureKind, gitwrap, spawn};
use crate::error::Error;
use crate::procenv::{InheritedEnv, Pairs};
use crate::store::records::Metrics;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use contract::AUTH_ENV;
#[cfg(test)]
pub(crate) use contract::TestOverrides;
pub(crate) use contract::{production_args, user_prompt};

#[cfg(test)]
mod tests;

pub(crate) struct ClaudeCli {
    env: Pairs,
    pub(super) real_git: PathBuf,
    pub(super) schemas: Schemas,
    #[cfg(test)]
    overrides: TestOverrides,
}

impl ClaudeCli {
    pub fn new(env: &InheritedEnv) -> Result<ClaudeCli, Error> {
        let path = env.get("PATH").unwrap_or_default();
        let real_git = gitwrap::find_git(path).ok_or(Error::NoGit)?;
        let mut vars = env.pick(&AUTH_ENV);
        vars.push(("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()));
        vars.push(("GIT_CONFIG_NOSYSTEM".into(), "1".into()));
        Ok(ClaudeCli {
            env: vars,
            real_git,
            schemas: Schemas::load()?,
            #[cfg(test)]
            overrides: TestOverrides::default(),
        })
    }

    pub fn env_vars(&self) -> &[(OsString, OsString)] {
        &self.env
    }

    pub fn args_for(&self, request: &AgentRequest) -> Vec<OsString> {
        contract::adjust(self, production_args(request, &self.schemas.cli_arg))
    }

    pub(super) fn run_env(&self, bin: &Path) -> Result<Pairs, Error> {
        let current = self.env_vars().iter().find(|(name, _)| name == "PATH");
        let current = current.map(|(_, value)| value.clone()).unwrap_or_default();
        let dirs = std::iter::once(bin.to_path_buf()).chain(std::env::split_paths(&current));
        let path = std::env::join_paths(dirs).map_err(std::io::Error::other);
        let path = path.map_err(Error::io("build the agent PATH"))?;
        let mut env: Pairs = self
            .env
            .iter()
            .filter(|(name, _)| name != "PATH")
            .cloned()
            .collect();
        env.push(("PATH".into(), path));
        Ok(env)
    }

    #[cfg(test)]
    pub fn with_overrides(mut self, overrides: TestOverrides) -> ClaudeCli {
        for (name, value) in &overrides.extra_env {
            self.env.retain(|(have, _)| have != name);
            self.env.push((name.clone(), value.clone()));
        }
        self.overrides = overrides;
        self
    }
}

impl Agent for ClaudeCli {
    async fn run(&self, request: &AgentRequest) -> AgentOutcome {
        match Box::pin(spawn::invoke(self, request)).await {
            Ok(captured) => classify(&captured, &self.schemas),
            Err(detail) => AgentOutcome::Failed {
                kind: FailureKind::CliError,
                detail: *detail,
                metrics: Metrics::default(),
            },
        }
    }
}
