use crate::agent::claude::ClaudeCli;
use crate::app::AppConfig;
use crate::cancel::StopReason;
use crate::error::Error;
use crate::gh::GhCli;
use crate::git::GitCli;
use crate::pr_url::{self, PrRef};
use crate::recovery;
use crate::render;
use crate::runner::{self, Deps, ReviewResult};
use crate::store::Store;
use std::process::ExitCode;
use tokio::sync::watch;

mod print;
pub(crate) mod signals;
mod wait;

pub(crate) async fn run(config: &AppConfig, url: &str, force: bool) -> ExitCode {
    let Ok(pr) = pr_url::parse(url) else {
        eprintln!(
            "error: not a github.com pull request URL: {}",
            render::sanitize(url)
        );
        return ExitCode::from(2);
    };
    match execute(config, &pr, force).await {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {}", render::sanitize(&error.to_string()));
            ExitCode::FAILURE
        }
    }
}

async fn execute(config: &AppConfig, pr: &PrRef, force: bool) -> Result<ExitCode, Error> {
    let deps = Deps {
        git: GitCli::new(&config.env),
        gh: GhCli::new(&config.env),
        agent: ClaudeCli::new(&config.env)?,
        settings: config.settings.clone(),
    };
    let store = Store::open(&config.paths).await?;
    let request = crate::store::jobs::EnqueueRequest {
        origin: crate::store::records::JobOrigin::Manual,
        force,
    };
    let enqueued = store.enqueue_review(pr, &config.process, request).await?;
    let job = enqueued.job;
    if enqueued.attached {
        eprintln!("attached to job {}", job.job_id);
    }
    let mut signals = signals::Signals::new()?;
    let _lock = loop {
        let lock = match wait::until_lock_or_end(
            &config.paths.lock(),
            &store,
            job.job_id,
            &pr.label(),
            &mut signals,
        )
        .await?
        {
            wait::Waited::Locked(lock) => lock,
            wait::Waited::Finished => return print::stored(&store, job.job_id, pr).await,
            wait::Waited::Stopped => return Ok(ExitCode::FAILURE),
        };
        lock.announce(crate::lock::Role::Review).await?;
        let report = recovery::run(&config.paths, &store, &deps.git, &config.settings).await?;
        for line in report.lines() {
            eprintln!("{}", render::sanitize(&line));
        }
        let stored = store.stored_result(job.job_id).await?;
        if stored.state != "queued" {
            return print::result(pr, &ReviewResult::from_stored(stored, false));
        }
        if !store
            .job_delayed(job.job_id, crate::store::now_ms())
            .await?
        {
            break lock;
        }
    };
    let (stop, cancel) = watch::channel(false);
    let paths = &config.paths;
    let job_run = runner::execute_review_job(paths, &store, &deps, pr, &job, &cancel);
    let result = signals::until_done(
        stop,
        job_run,
        &store,
        job.job_id,
        StopReason::UserCancel,
        &mut signals,
    )
    .await?;
    print::result(pr, &result)
}
