mod body;
mod checks;
mod gate;
mod load;
mod outcome;
mod post;
mod reconcile;
mod sync;

#[cfg(test)]
mod tests;

pub(crate) use post::execute_publish;
#[cfg(test)]
pub(crate) use post::execute_publish_at;
pub(crate) use reconcile::execute_reconcile;
pub(crate) use sync::execute_sync;

pub(crate) async fn serve_next<H: crate::gh::Gh>(
    store: &crate::store::Store,
    gh: &H,
    max_retries: u32,
) -> Result<bool, crate::error::Error> {
    let Some(job) = store.next_service_job(crate::store::now_ms()).await? else {
        return Ok(false);
    };
    if store.start_job(job.id).await?.is_some() {
        match job.kind.as_str() {
            "publish" => execute_publish(store, gh, job.id, max_retries).await?,
            "reconcile" => execute_reconcile(store, gh, job.id).await?,
            "sync" => execute_sync(store, gh, job.id).await?,
            _ => return Err(rusqlite::Error::InvalidQuery.into()),
        }
    }
    Ok(true)
}
