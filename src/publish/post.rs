use super::body::from_snapshot;
use super::load::load_job;
use crate::error::Error;
use crate::gh::{Gh, ReviewRequest};
use crate::pr_url::PrRef;
use crate::store::Store;
use rusqlite::params;

pub(crate) async fn execute_publish(
    store: &Store,
    gh: &impl Gh,
    job_id: i64,
    max_retries: u32,
) -> Result<(), Error> {
    execute(store, gh, job_id, max_retries, &crate::crash::Off).await
}

#[cfg(test)]
pub(crate) async fn execute_publish_at(
    store: &Store,
    gh: &impl Gh,
    job_id: i64,
    max_retries: u32,
    probe: &dyn crate::crash::Probe,
) -> Result<(), Error> {
    execute(store, gh, job_id, max_retries, probe).await
}

async fn execute(
    store: &Store,
    gh: &impl Gh,
    job_id: i64,
    max_retries: u32,
    probe: &dyn crate::crash::Probe,
) -> Result<(), Error> {
    let job = load_job(store, job_id).await?;
    if !super::gate::before_send(store, gh, &job, job_id, max_retries).await? {
        return Ok(());
    }
    crate::crash::at("publish-before-sending", probe).await;
    mark_sending(store, &job.publication_id).await?;
    crate::crash::at("publish-after-sending", probe).await;
    let (body, comments) = from_snapshot(&job.snapshot, &job.publication_id)?;
    let request = ReviewRequest {
        commit_id: job.head_sha,
        body,
        event: job.event,
        comments,
    };
    let pr = PrRef {
        owner: job.owner,
        repo: job.repo,
        number: job.number,
    };
    super::outcome::finish(store, gh, &pr, &request, job_id, &job.publication_id).await
}

async fn mark_sending(store: &Store, publication_id: &str) -> Result<(), Error> {
    let publication_id = publication_id.to_string();
    let now = crate::store::now_ms();
    let changed = store
        .call(move |conn| {
            let sql = "UPDATE publications SET state = 'sending', updated_at = ?2 WHERE id = ?1 AND state = 'pending'";
            conn.execute(sql, params![publication_id, now])
        })
        .await?;
    if changed == 0 {
        return Err(stuck("publication is not pending"));
    }
    Ok(())
}

fn stuck(detail: &str) -> Error {
    Error::Command {
        program: "gh",
        args: "publish".into(),
        detail: detail.into(),
    }
}
