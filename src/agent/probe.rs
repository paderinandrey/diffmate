use super::FailureDetail;
use crate::proc::excerpt;
use std::ffi::OsString;

#[cfg(test)]
mod live_tests;
mod run;
#[cfg(test)]
mod tests;

pub(crate) use run::{check_flags, check_version};

pub(crate) const MIN_VERSION: (u32, u32, u32) = (2, 1, 304);
pub(crate) const PROBE_OK_TEXT: &str =
    "Error: Input must be provided either through stdin or as a prompt argument when using --print";
const UNKNOWN_OPTION: &str = "error: unknown option '";
const SCHEMA_REJECTED: &str = "--json-schema is not a valid JSON Schema";
const PROMPT_MISSING: &str = "Append system prompt file not found";

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
    let word = stdout.lines().next()?.split_whitespace().next()?;
    let number = |part: &str| {
        let digits = !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
        digits.then(|| part.parse().ok()).flatten()
    };
    let mut parts = word.split('.').map(number);
    let triple = (parts.next()??, parts.next()??, parts.next()??);
    parts.next().is_none().then_some(triple)
}

pub(crate) fn interpret_probe(
    stdout: &[u8],
    stderr: &[u8],
    exit_code: Option<i32>,
) -> Result<(), Refusal> {
    let text = String::from_utf8_lossy(stderr);
    let refused = |message: String| refuse(message, exit_code, stderr);
    if let Some((_, rest)) = text.split_once(UNKNOWN_OPTION) {
        let flag: String = rest.chars().take_while(|c| *c != '\'').take(80).collect();
        return Err(refused(format!("Claude CLI does not support {flag}")));
    }
    if text.contains(SCHEMA_REJECTED) {
        return Err(refused(
            "Claude CLI rejected the --json-schema argument".into(),
        ));
    }
    if text.contains(PROMPT_MISSING) {
        return Err(refused("Claude CLI cannot read the prompt file".into()));
    }
    if text.contains(PROBE_OK_TEXT) && exit_code.is_some_and(|code| code != 0) {
        return Ok(());
    }
    let seen = excerpt(stdout, 100);
    Err(refused(format!(
        "flag probe inconclusive: exit code {exit_code:?}, stdout {seen:?}"
    )))
}
