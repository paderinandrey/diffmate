use super::Notice;
use crate::error::Error;
use crate::store::records::JobOrigin;
use crate::store::{JobId, Store};

pub(crate) async fn notices(store: &Store, jobs: &[(JobId, String)]) -> Result<Vec<Notice>, Error> {
    let mut due = Vec::new();
    for (job, text) in jobs {
        if !text.starts_with("failed") {
            continue;
        }
        let (origin, _) = store.job_retry_state(*job).await?;
        if origin != JobOrigin::Automatic {
            continue;
        }
        due.push(Notice::GaveUp {
            label: store.job_label(*job).await?,
            reason: text.clone(),
        });
    }
    Ok(due)
}
