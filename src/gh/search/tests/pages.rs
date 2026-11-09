use super::super::search_review_requested;
use crate::error::Error;
use std::future::ready;

fn pulls(count: u64, origin: u64) -> Vec<u8> {
    let values = (0..count)
        .map(|offset| {
            super::pull(
                origin + offset,
                "acme/repo",
                "acme",
                "repo",
                offset == 0,
                "OPEN",
            )
        })
        .collect();
    serde_json::to_vec(&serde_json::Value::Array(values)).unwrap()
}

fn blanks(count: usize) -> Vec<u8> {
    format!("[{}]", vec!["{}"; count].join(",")).into_bytes()
}

struct Script {
    pages: Vec<Result<Vec<u8>, Error>>,
    calls: Vec<u32>,
}

async fn run(
    pages: Vec<Result<Vec<u8>, Error>>,
) -> (Vec<u32>, Result<Vec<crate::gh::DirectRequest>, Error>) {
    let mut script = Script {
        pages,
        calls: Vec::new(),
    };
    let result = search_review_requested(&mut script, |script, limit| {
        script.calls.push(limit);
        ready(script.pages.remove(0))
    })
    .await;
    (script.calls, result)
}

#[tokio::test]
async fn a_full_page_is_replaced_by_the_next_shorter_page() {
    let (calls, result) = run(vec![Ok(pulls(100, 1)), Ok(pulls(40, 1_000))]).await;
    let kept = result.unwrap();
    assert_eq!(calls, [100, 200]);
    assert_eq!(kept.len(), 40);
    assert_eq!(kept[0].pr.number, 1_000);
    assert_eq!(kept[39].pr.number, 1_039);
    assert!(kept[0].is_draft);
}

#[tokio::test]
async fn a_short_page_ends_after_one_call() {
    let (calls, result) = run(vec![Ok(pulls(30, 1))]).await;
    let kept = result.unwrap();
    assert_eq!(calls, [100]);
    assert_eq!(kept.len(), 30);
    assert!(kept[0].is_draft);
}

#[tokio::test]
async fn a_failed_second_page_discards_the_first() {
    let error = Error::Command {
        program: "gh",
        args: "search prs".into(),
        detail: "unavailable".into(),
    };
    let (calls, result) = run(vec![Ok(pulls(100, 1)), Err(error)]).await;
    assert_eq!(calls, [100, 200]);
    assert!(matches!(
        result,
        Err(Error::Command { detail, .. }) if detail == "unavailable"
    ));
}

#[tokio::test]
async fn a_full_thousand_stops_without_another_call() {
    let mut calls = Vec::new();
    let kept = search_review_requested(&mut calls, |calls, limit| {
        calls.push(limit);
        ready(Ok(blanks(usize::try_from(limit).unwrap())))
    })
    .await
    .unwrap();
    assert_eq!(kept.len(), 0);
    assert_eq!(calls, [100, 200, 300, 400, 500, 600, 700, 800, 900, 1_000]);
}
