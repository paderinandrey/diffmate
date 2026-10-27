use crate::app::AppConfig;
use crate::error::Error;
use crate::render::sanitize;
use crate::store::Store;
use std::process::ExitCode;

fn line(parts: [&str; 3]) -> String {
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
    let rows = targets.iter();
    Ok(rows
        .map(|t| line([&t.label, &t.tracking, &t.latest]))
        .collect())
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
