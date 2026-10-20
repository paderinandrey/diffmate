use crate::agent::claude::ClaudeCli;
use crate::app::AppConfig;
use crate::error::Error;
use crate::gh::GhCli;
use crate::git::GitCli;
use crate::pr_url::{self, PrRef};
use crate::render;
use crate::runner::{self, Deps, ReviewResult};
use crate::store::Store;
use std::process::ExitCode;

pub(crate) async fn run(config: &AppConfig, url: &str) -> ExitCode {
    let Ok(pr) = pr_url::parse(url) else {
        eprintln!(
            "error: not a github.com pull request URL: {}",
            render::sanitize(url)
        );
        return ExitCode::from(2);
    };
    match execute(config, &pr).await {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {}", render::sanitize(&error.to_string()));
            ExitCode::FAILURE
        }
    }
}

async fn execute(config: &AppConfig, pr: &PrRef) -> Result<ExitCode, Error> {
    let deps = Deps {
        git: GitCli::new(&config.env),
        gh: GhCli::new(&config.env),
        agent: ClaudeCli::new(&config.env)?,
    };
    let store = Store::open(&config.paths).await?;
    let job = store.enqueue_review(pr, &config.process).await?;
    let result = runner::execute_review_job(&config.paths, &store, &deps, pr, &job).await;
    let (stdout, stderr) = (std::io::stdout(), std::io::stderr());
    let printed = render::review(pr, &result, &mut stdout.lock(), &mut stderr.lock());
    printed.map_err(Error::io("write the result"))?;
    let succeeded = matches!(result, ReviewResult::Succeeded { .. });
    Ok(if succeeded {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}
