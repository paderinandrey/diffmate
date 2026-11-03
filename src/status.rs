use crate::app::AppConfig;
use crate::error::Error;
use crate::gh::{Gh, GhCli};
use crate::render::sanitize;
use crate::store::{Store, now_ms};
use freshness::LiveHead;
use std::process::ExitCode;

mod freshness;
#[cfg(test)]
mod tests;
mod when;

fn line<const N: usize>(parts: [&str; N]) -> String {
    let clean = parts.map(|part| sanitize(part).replace(['\n', '\t'], " "));
    clean.join("  ")
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
    let mut rows = Vec::with_capacity(targets.len());
    for target in targets {
        let mut row = line([&target.label, &target.tracking, &target.latest]);
        if let Some(draft) = &target.draft_head {
            let live = match gh.pr_view(&target.pr).await {
                Ok(meta) => LiveHead::Fetched(meta.head_sha),
                Err(error) => {
                    let warning = format!(
                        "warning: live head check for {} failed: {error}",
                        target.label
                    );
                    eprintln!("{}", line([&warning]));
                    LiveHead::Failed
                }
            };
            let known = target
                .known_head
                .as_ref()
                .map(|(head, time)| (head.as_str(), *time));
            row = line([&row, &freshness::describe(draft, &live, known, now_ms())]);
        }
        rows.push(row);
    }
    Ok(rows)
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
