use super::ClaudeCli;
use crate::agent::AgentRequest;
use std::ffi::OsString;
use std::path::Path;

pub(crate) const AUTH_ENV: [&str; 15] = [
    "PATH",
    "HOME",
    "USER",
    "LOGNAME",
    "TMPDIR",
    "LANG",
    "LC_ALL",
    "LC_CTYPE",
    "XAI_API_KEY",
    "GROK_CODE_XAI_API_KEY",
    "GROK_HOME",
    "HTTPS_PROXY",
    "HTTP_PROXY",
    "NO_PROXY",
    "SSL_CERT_FILE",
];

pub(crate) const LOCKED_ENV: &[(&str, &str)] = &[
    ("GROK_CLAUDE_AGENTS_ENABLED", "0"),
    ("GROK_CLAUDE_HOOKS_ENABLED", "0"),
    ("GROK_CLAUDE_MCPS_ENABLED", "0"),
    ("GROK_CLAUDE_RULES_ENABLED", "0"),
    ("GROK_CLAUDE_SKILLS_ENABLED", "0"),
    ("GROK_CURSOR_AGENTS_ENABLED", "0"),
    ("GROK_CURSOR_HOOKS_ENABLED", "0"),
    ("GROK_CURSOR_MCPS_ENABLED", "0"),
    ("GROK_CURSOR_RULES_ENABLED", "0"),
    ("GROK_CURSOR_SKILLS_ENABLED", "0"),
];

const DISALLOWED_TOOLS: &str = "run_terminal_cmd,write,search_replace,web_search,web_fetch,task,Agent,search_tool,use_tool,todo_write";

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

pub(crate) fn production_args(
    request: &AgentRequest,
    cli_schema: &str,
    rules: &str,
) -> Vec<OsString> {
    let mut args = vec![
        OsString::from("-p"),
        user_prompt(&request.context_dir).into(),
    ];
    let flags = [
        "--tools",
        "read_file,grep,list_dir",
        "--disallowed-tools",
        DISALLOWED_TOOLS,
        "--disable-web-search",
        "--no-subagents",
        "--sandbox",
        "read-only",
        "--permission-mode",
        "dontAsk",
        "--output-format",
        "streaming-messages-json",
        "--json-schema",
    ];
    args.extend(flags.map(OsString::from));
    args.push(cli_schema.into());
    args.push("--rules".into());
    args.push(rules.into());
    args
}

pub(crate) fn user_prompt(context_dir: &Path) -> String {
    format!(
        "Review the pull request whose context files are in {}. Follow the review rules and return only the JSON document they describe.",
        context_dir.display()
    )
}
