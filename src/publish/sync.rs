mod apply;

use crate::error::Error;
use crate::gh::Gh;
use crate::pr_url::PrRef;
use crate::store::Store;
use rusqlite::params;

const DELAY_MS: i64 = 60_000;

struct Loaded {
    publication_id: String,
    review_id: String,
    pr: PrRef,
}

pub(crate) async fn execute_sync(store: &Store, gh: &impl Gh, job_id: i64) -> Result<(), Error> {
    let loaded = load(store, job_id).await?;
    let Ok(comments) = gh
        .review_comment_threads(&loaded.pr, &loaded.review_id)
        .await
    else {
        return later(store, job_id).await;
    };
    let links = apply::matched(store, &loaded.publication_id, &comments).await?;
    let statuses = if links.is_empty() {
        None
    } else {
        gh.thread_resolved(&loaded.pr).await.ok()
    };
    apply::store_links(store, job_id, &loaded.publication_id, links, statuses).await
}

async fn load(store: &Store, job_id: i64) -> Result<Loaded, Error> {
    store
        .call(move |conn| {
            let sql = "SELECT p.id, p.github_review_id, t.owner, t.repo, t.number FROM jobs j \
                       JOIN publications p ON p.id = j.publication_id \
                       JOIN targets t ON t.id = j.target_id \
                       WHERE j.id = ?1 AND j.kind = 'sync' AND p.state = 'submitted'";
            conn.query_row(sql, [job_id], |row| {
                let number: i64 = row.get(4)?;
                let number = u64::try_from(number)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
                Ok(Loaded {
                    publication_id: row.get(0)?,
                    review_id: row.get(1)?,
                    pr: PrRef {
                        owner: row.get(2)?,
                        repo: row.get(3)?,
                        number,
                    },
                })
            })
        })
        .await
}

async fn later(store: &Store, job_id: i64) -> Result<(), Error> {
    let now = crate::store::now_ms();
    store
        .call(move |conn| {
            let sql = "UPDATE jobs SET state = 'queued', next_attempt_at = ?2, updated_at = ?3 \
                       WHERE id = ?1 AND kind = 'sync'";
            if conn.execute(sql, params![job_id, now + DELAY_MS, now])? == 1 {
                Ok(())
            } else {
                Err(apply::missing())
            }
        })
        .await
}
