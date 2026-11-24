use crate::app::AppConfig;
use crate::error::Error;
use crate::gh::{Gh, GhCli};
use crate::pr_url::PrRef;
use crate::render::sanitize;
use crate::store::status::TargetStatus;
use crate::store::{Store, now_ms};
use freshness::LiveHead;
use std::borrow::Cow;
use std::cell::RefCell;
use std::process::ExitCode;
use tokio::time::{Duration, timeout};

mod budget;
pub(crate) mod freshness;
#[cfg(test)]
mod tests;
mod when;

const HEAD_TIMEOUT: Duration = Duration::from_secs(5);

fn line<const N: usize>(parts: [&str; N]) -> String {
    let clean = parts.map(|part| sanitize(part).replace(['\n', '\t'], " "));
    clean.join("  ")
}

fn latest(target: &TargetStatus, now_ms: i64) -> Cow<'_, str> {
    match budget::describe(&target.retry, now_ms) {
        Some(retry) => Cow::Owned(format!("{}; {retry}", target.latest)),
        None => Cow::Borrowed(&target.latest),
    }
}

async fn live_head(
    target: &TargetStatus,
    read_head: &impl AsyncFn(&PrRef) -> Result<String, Error>,
) -> LiveHead {
    let detail = match timeout(HEAD_TIMEOUT, read_head(&target.pr)).await {
        Ok(Ok(head)) => return LiveHead::Fetched(head),
        Ok(Err(error)) => error.to_string(),
        Err(_) => format!("timed out after {}s", HEAD_TIMEOUT.as_secs()),
    };
    let warning = format!(
        "warning: live head check for {} failed: {detail}",
        target.label
    );
    eprintln!("{}", line([&warning]));
    LiveHead::Failed
}

async fn target_line(
    target: &TargetStatus,
    read_head: &impl AsyncFn(&PrRef) -> Result<String, Error>,
) -> String {
    let latest = latest(target, now_ms());
    let Some(draft) = &target.draft_head else {
        return line([
            &target.label,
            &target.tracking,
            &latest,
            &target.publication,
        ]);
    };
    let live = live_head(target, read_head).await;
    let known = target
        .known_head
        .as_ref()
        .map(|(sha, at)| (sha.as_str(), *at));
    let freshness = freshness::describe(draft, &live, known, now_ms());
    line([
        &target.label,
        &target.tracking,
        &latest,
        &freshness,
        &target.publication,
    ])
}

async fn target_lines(
    targets: &[TargetStatus],
    read_head: impl AsyncFn(&PrRef) -> Result<String, Error>,
) -> Vec<String> {
    let mut rows = vec![String::new(); targets.len()];
    let pending = RefCell::new(targets.iter().zip(&mut rows));
    let next = || pending.borrow_mut().next();
    let worker = async || {
        while let Some((target, row)) = next() {
            *row = target_line(target, &read_head).await;
        }
    };
    tokio::join!(worker(), worker(), worker());
    rows
}

async fn lines(config: &AppConfig) -> Result<Vec<String>, Error> {
    let Some(store) = Store::open_read_only(&config.paths).await? else {
        return Ok(vec!["No targets.".to_string()]);
    };
    let targets = store.target_statuses().await?;
    if targets.is_empty() {
        return Ok(vec!["No targets.".to_string()]);
    }
    let gh = GhCli::new(&config.env);
    Ok(target_lines(&targets, async |pr| Ok(gh.pr_view(pr).await?.head_sha)).await)
}

pub(crate) async fn run(config: &AppConfig) -> ExitCode {
    match lines(config).await {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {}", sanitize(&error.to_string()));
            ExitCode::FAILURE
        }
    }
}
