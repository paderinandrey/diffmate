use super::checks;
use super::load::JobRow;
use crate::error::Error;
use crate::gh::Gh;
use crate::pr_url::PrRef;
use crate::store::Store;
use std::future::Future;
use tokio::time::{Duration, sleep, timeout};

const CHECK_TIMEOUT: Duration = Duration::from_secs(5);

pub(super) async fn before_send(
    store: &Store,
    gh: &impl Gh,
    job: &JobRow,
    job_id: i64,
    max_retries: u32,
) -> Result<bool, Error> {
    if job.publication_state == "abandoned" {
        store
            .fail_job(job_id, "publication is abandoned".into())
            .await?;
        return Ok(false);
    }
    let pr = PrRef {
        owner: job.owner.clone(),
        repo: job.repo.clone(),
        number: job.number,
    };
    let head = match retry("HEAD check", max_retries, || gh.pr_view(&pr)).await {
        Ok(meta) => meta.head_sha,
        Err(reason) => return stop(store, job_id, &job.publication_id, &reason).await,
    };
    if head != job.confirmed_sha {
        store.mark_needs_confirmation(job_id, &head).await?;
        return Ok(false);
    }
    let login = match retry("login check", max_retries, || gh.current_login()).await {
        Ok(login) => login,
        Err(reason) => return stop(store, job_id, &job.publication_id, &reason).await,
    };
    let reviews = match retry("list reviews", max_retries, || gh.list_my_reviews(&pr)).await {
        Ok(reviews) => reviews,
        Err(reason) => return stop(store, job_id, &job.publication_id, &reason).await,
    };
    if checks::pending_review_of_mine(&reviews, &login) {
        let reason = "you already have a pending review on this pull request";
        return stop(store, job_id, &job.publication_id, reason).await;
    }
    Ok(true)
}

async fn stop(
    store: &Store,
    job_id: i64,
    publication_id: &str,
    reason: &str,
) -> Result<bool, Error> {
    store
        .abandon_pending(publication_id, job_id, reason)
        .await?;
    Ok(false)
}

async fn retry<T, F, Fut>(check: &str, max_retries: u32, mut call: F) -> Result<T, String>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, Error>>,
{
    let mut attempt = 0u32;
    loop {
        match timeout(CHECK_TIMEOUT, call()).await {
            Ok(Ok(value)) => return Ok(value),
            Ok(Err(error)) if attempt == max_retries || !checks::transient_gh(&error) => {
                return Err(checks::failed_check(check, &error));
            }
            Err(_) if attempt == max_retries => return Err(format!("{check} timed out")),
            _ => {}
        }
        attempt += 1;
        sleep(Duration::from_secs(if attempt == 1 { 1 } else { 5 })).await;
    }
}
