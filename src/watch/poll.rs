use crate::app::ProcessInfo;
use crate::error::Error;
use crate::gh::{DirectRequest, Gh, PrMeta};
use crate::pr_url::PrRef;
use crate::store::poll::{Absent, KnownTarget, SeenRequest};
use crate::store::{Store, now_ms};
use std::time::Duration;

pub(super) async fn once<H: Gh>(
    gh: &H,
    store: &Store,
    debounce: Duration,
    process: &ProcessInfo,
) -> Result<(), Error> {
    let hits = gh.search_review_requested().await?;
    let known = store.known_targets().await?;
    let absent = missing(gh, &known, &hits).await;
    let seen = present(gh, &known, &hits).await;
    let debounce_ms = i64::try_from(debounce.as_millis()).unwrap_or(i64::MAX);
    store
        .apply_poll(seen, absent, now_ms(), debounce_ms, process)
        .await
}

async fn missing<H: Gh>(
    gh: &H,
    known: &[KnownTarget],
    hits: &[DirectRequest],
) -> Vec<(PrRef, Absent)> {
    let mut rows = Vec::new();
    for target in known {
        if hits.iter().any(|hit| hit.pr == target.pr) {
            continue;
        }
        rows.push((target.pr.clone(), classify(gh.pr_view(&target.pr).await)));
    }
    rows
}

fn classify(view: Result<PrMeta, Error>) -> Absent {
    match view {
        Err(_) => Absent::Skip,
        Ok(meta) if closed(&meta.state) => Absent::Closed,
        Ok(meta) => Absent::Open {
            is_draft: meta.is_draft,
            head_sha: Some(meta.head_sha),
        },
    }
}

fn closed(state: &str) -> bool {
    state.eq_ignore_ascii_case("closed") || state.eq_ignore_ascii_case("merged")
}

async fn present<H: Gh>(gh: &H, known: &[KnownTarget], hits: &[DirectRequest]) -> Vec<SeenRequest> {
    let mut rows = Vec::new();
    for hit in hits {
        let head_sha = if wanted(known, hit) {
            gh.pr_view(&hit.pr).await.ok().map(|meta| meta.head_sha)
        } else {
            None
        };
        rows.push(SeenRequest {
            pr: hit.pr.clone(),
            is_draft: hit.is_draft,
            head_sha,
        });
    }
    rows
}

fn wanted(known: &[KnownTarget], hit: &DirectRequest) -> bool {
    !hit.is_draft
        && !known
            .iter()
            .any(|row| row.pr == hit.pr && row.tracking != "tracked" && row.request_seen)
}
