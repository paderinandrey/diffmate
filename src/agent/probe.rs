use super::FailureDetail;
use crate::proc::excerpt;
use std::ffi::OsString;

#[cfg(test)]
mod live_tests;
mod run;
#[cfg(test)]
mod tests;

pub(crate) use run::{check_flags, check_version};

pub(crate) const MIN_VERSION: (u32, u32, u32) = (1, 0, 77);
pub(crate) const HELP_FLAGS: &[&str] = &[
    "--single",
    "--tools",
    "--disallowed-tools",
    "--disable-web-search",
    "--no-subagents",
    "--sandbox",
    "--permission-mode",
    "--output-format",
    "--json-schema",
    "--rules",
];

type Env = [(OsString, OsString)];
pub(crate) type Refusal = Box<FailureDetail>;

fn refuse(message: String, exit_code: Option<i32>, stderr: &[u8]) -> Refusal {
    let stderr_excerpt = Some(excerpt(stderr, 500)).filter(|text| !text.is_empty());
    Box::new(FailureDetail {
        message,
        exit_code,
        stderr_excerpt,
        ..FailureDetail::default()
    })
}

fn dotted((major, minor, patch): (u32, u32, u32)) -> String {
    format!("{major}.{minor}.{patch}")
}

pub(crate) fn parse_version(stdout: &str) -> Option<(u32, u32, u32)> {
    let number = |part: &str| {
        let digits = !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
        digits.then(|| part.parse().ok()).flatten()
    };
    let triple = |word: &str| {
        let mut parts = word.split('.').map(number);
        let found = (parts.next()??, parts.next()??, parts.next()??);
        parts.next().is_none().then_some(found)
    };
    stdout.lines().next()?.split_whitespace().find_map(triple)
}

pub(crate) fn interpret_help(stdout: &[u8], exit_code: Option<i32>) -> Result<(), Refusal> {
    let text = String::from_utf8_lossy(stdout);
    let refused = |message: String| refuse(message, exit_code, stdout);
    if exit_code == Some(0) {
        if let Some(flag) = HELP_FLAGS.iter().find(|flag| !text.contains(*flag)) {
            return Err(refused(format!("Grok CLI does not support {flag}")));
        }
        return Ok(());
    }
    let seen = excerpt(stdout, 100);
    Err(refused(format!(
        "flag probe inconclusive: exit code {exit_code:?}, stdout {seen:?}"
    )))
}
