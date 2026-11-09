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
    let mut absent = missing(gh, &known, &hits).await;
    let (seen, later) = present(gh, &known, &hits).await;
    absent.extend(later);
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
        Ok(meta) if closed(&meta.state) => Absent::Closed {
            head_sha: Some(meta.head_sha),
        },
        Ok(meta) => Absent::Open {
            is_draft: meta.is_draft,
            head_sha: Some(meta.head_sha),
        },
    }
}

fn closed(state: &str) -> bool {
    state.eq_ignore_ascii_case("closed") || state.eq_ignore_ascii_case("merged")
}

async fn present<H: Gh>(
    gh: &H,
    known: &[KnownTarget],
    hits: &[DirectRequest],
) -> (Vec<SeenRequest>, Vec<(PrRef, Absent)>) {
    let mut seen = Vec::new();
    let mut later = Vec::new();
    for hit in hits {
        if !wanted(known, hit) {
            seen.push(from_search(hit));
            continue;
        }
        match gh.pr_view(&hit.pr).await {
            Err(_) => seen.push(from_search(hit)),
            Ok(meta) if closed(&meta.state) => later.push((
                hit.pr.clone(),
                Absent::Closed {
                    head_sha: Some(meta.head_sha),
                },
            )),
            Ok(meta) => seen.push(SeenRequest {
                pr: hit.pr.clone(),
                is_draft: meta.is_draft,
                head_sha: Some(meta.head_sha),
            }),
        }
    }
    (seen, later)
}

fn from_search(hit: &DirectRequest) -> SeenRequest {
    SeenRequest {
        pr: hit.pr.clone(),
        is_draft: hit.is_draft,
        head_sha: None,
    }
}

fn wanted(known: &[KnownTarget], hit: &DirectRequest) -> bool {
    !hit.is_draft
        && !known
            .iter()
            .any(|row| row.pr == hit.pr && row.tracking != "tracked" && row.request_seen)
}
