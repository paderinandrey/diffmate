use super::ClaudeCli;
use crate::agent::AgentRequest;
use std::ffi::OsString;
use std::path::Path;

pub(super) const AUTH_ENV: [&str; 17] = [
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

#[cfg(test)]
#[derive(Default)]
pub(crate) struct TestOverrides {
    pub extra_args: Vec<OsString>,
    pub removed_flags: Vec<&'static str>,
    pub extra_env: Vec<(OsString, OsString)>,
}

#[cfg(test)]
impl TestOverrides {
    fn apply(&self, mut args: Vec<OsString>) -> Vec<OsString> {
        args.retain(|arg| {
            !self
                .removed_flags
                .iter()
                .any(|flag| arg.to_str() == Some(*flag))
        });
        args.extend(self.extra_args.iter().cloned());
        args
    }
}

#[cfg(test)]
pub(super) fn adjust(cli: &ClaudeCli, args: Vec<OsString>) -> Vec<OsString> {
    cli.overrides.apply(args)
}

#[cfg(not(test))]
pub(super) fn adjust(_: &ClaudeCli, args: Vec<OsString>) -> Vec<OsString> {
    args
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
