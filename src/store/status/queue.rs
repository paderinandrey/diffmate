use super::{TargetStatus, read_status};
use crate::error::Error;
use crate::pr_url::PrRef;
use crate::status::freshness;
use crate::store::{Store, TargetId, now_ms};

pub(crate) struct QueueRow {
    pub label: String,
    pub tracking: String,
    pub latest: String,
    pub freshness: String,
    pub publication: String,
    pub target_id: TargetId,
    pub pr: PrRef,
}

fn freshness_of(target: &TargetStatus, now: i64) -> String {
    let Some(draft) = target.draft_head.as_deref() else {
        return String::new();
    };
    let known = target
        .known_head
        .as_ref()
        .map(|(head, checked)| (head.as_str(), *checked));
    freshness::stored(draft, known, now)
}

fn queue_row(target: TargetStatus, now: i64) -> QueueRow {
    let freshness = freshness_of(&target, now);
    QueueRow {
        label: target.label,
        tracking: target.tracking,
        latest: target.latest,
        freshness,
        publication: target.publication,
        target_id: target.id,
        pr: target.pr,
    }
}

impl Store {
    pub async fn queue_rows(&self) -> Result<Vec<QueueRow>, Error> {
        let now = now_ms();
        self.call(move |conn| {
            let sql = format!("{} ORDER BY t.id DESC", super::SQL);
            let mut statement = conn.prepare(&sql)?;
            let rows = statement.query_map([], read_status)?;
            rows.map(|row| row.map(|target| queue_row(target, now)))
                .collect()
        })
        .await
    }
}
