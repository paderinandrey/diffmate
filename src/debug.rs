use crate::app::AppConfig;
use crate::pr_url;
use crate::render;
use crate::store::{Store, jobs::EnqueueRequest, records::JobOrigin};
use std::process::ExitCode;

pub(crate) async fn enqueue(
    config: &AppConfig,
    url: &str,
    automatic: bool,
    force: bool,
) -> ExitCode {
    let Ok(pr) = pr_url::parse(url) else {
        eprintln!(
            "error: not a github.com pull request URL: {}",
            render::sanitize(url)
        );
        return ExitCode::from(2);
    };
    let result = async {
        let store = Store::open(&config.paths).await?;
        let origin = if automatic {
            JobOrigin::Automatic
        } else {
            JobOrigin::Manual
        };
        store
            .enqueue_review(&pr, &config.process, EnqueueRequest { origin, force })
            .await
    }
    .await;
    match result {
        Ok(enqueued) => {
            if enqueued.attached {
                println!("attached to job {}", enqueued.job.job_id);
            } else {
                println!("job {} queued", enqueued.job.job_id);
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {}", render::sanitize(&error.to_string()));
            ExitCode::FAILURE
        }
    }
}
