use super::envelope::classify;
use super::schema::Schemas;
use super::{Agent, AgentOutcome, AgentRequest, FailureDetail, FailureKind, gitwrap, spawn};
use crate::error::Error;
use crate::procenv::{InheritedEnv, Pairs};
use crate::store::records::Metrics;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

const AUTH_ENV: [&str; 17] = [
    "PATH",
    "HOME",
    "USER",
    "LOGNAME",
    "TMPDIR",
    "LANG",
    "LC_ALL",
    "LC_CTYPE",
    "CLAUDE_CONFIG_DIR",
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_AUTH_TOKEN",
    "CLAUDE_CODE_OAUTH_TOKEN",
    "HTTPS_PROXY",
    "HTTP_PROXY",
    "NO_PROXY",
    "NODE_EXTRA_CA_CERTS",
    "SSL_CERT_FILE",
];

pub(crate) struct ClaudeCli {
    pub(super) env: Pairs,
    pub(super) path: OsString,
    pub(super) real_git: PathBuf,
    pub(super) schemas: Schemas,
}

impl ClaudeCli {
    pub fn new(env: &InheritedEnv) -> Result<ClaudeCli, Error> {
        let path = env.get("PATH").unwrap_or_default().to_os_string();
        let real_git = gitwrap::find_git(&path).ok_or(Error::NoGit)?;
        let vars = env.pick(&AUTH_ENV).into_iter();
        Ok(ClaudeCli {
            env: vars.filter(|(name, _)| name != "PATH").collect(),
            path,
            real_git,
            schemas: Schemas::load()?,
        })
    }
}

impl Agent for ClaudeCli {
    async fn run(&self, request: &AgentRequest) -> AgentOutcome {
        match Box::pin(spawn::invoke(self, request)).await {
            Ok(captured) => classify(&captured, &self.schemas),
            Err(error) => AgentOutcome::Failed {
                kind: FailureKind::CliError,
                detail: FailureDetail {
                    message: error.to_string(),
                    ..FailureDetail::default()
                },
                metrics: Metrics::default(),
            },
        }
    }
}

pub(crate) fn production_args(request: &AgentRequest, cli_schema: &str) -> Vec<OsString> {
    let flags = [
        "-p",
        "--safe-mode",
        "--restricted",
        "--strict-mcp-config",
        "--tools",
        "Read,Grep,Glob",
        "--no-session-persistence",
        "--permission-prompts",
        "none",
        "--disable-slash-commands",
        "--add-dir",
    ];
    let mut args: Vec<OsString> = flags.iter().map(OsString::from).collect();
    args.push(request.context_dir.clone().into());
    args.push("--append-system-prompt-file".into());
    args.push(request.prompt_file.clone().into());
    args.extend(["--output-format", "json", "--json-schema"].map(OsString::from));
    args.push(cli_schema.into());
    args
}

pub(crate) fn user_prompt(context_dir: &Path) -> String {
    format!(
        "Review the pull request whose context files are in {}. Follow the review rules and return only the JSON document they describe.",
        context_dir.display()
    )
}
