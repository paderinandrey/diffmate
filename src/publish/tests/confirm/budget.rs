use super::RUN;
use super::boom::Boom;
use super::harness::{open_gh, pair};
use crate::error::Error;
use std::time::Duration;

fn retries() -> u32 {
    crate::config::Settings::default().max_retries()
}

fn cmd(detail: &str) -> Error {
    Error::Command {
        program: "gh",
        args: String::new(),
        detail: detail.into(),
    }
}

async fn queued(store: &crate::store::Store) -> i64 {
    store
        .confirm_publication(1, "COMMENT", RUN, &super::super::support::process())
        .await
        .unwrap()
}

async fn run(view: bool, detail: &'static str) -> (Boom, Duration, (String, String), String) {
    let (_tmp, store, _, _) = open_gh(RUN).await;
    let gh = Boom::new(detail, view);
    let start = tokio::time::Instant::now();
    crate::publish::execute_publish(&store, &gh, queued(&store).await, retries())
        .await
        .unwrap();
    let reason = super::super::support::cell(&store, "SELECT reason FROM publications").await;
    (gh, start.elapsed(), pair(&store).await, reason)
}

#[tokio::test(start_paused = true)]
async fn transient_checks_sleep_then_abandon() {
    for view in [true, false] {
        let detail = if view { "HTTP 503" } else { "HTTP 429" };
        let (gh, elapsed, state, reason) = run(view, detail).await;
        assert_eq!(elapsed, Duration::from_secs(6), "{detail}");
        assert_eq!(gh.views(), if view { 3 } else { 1 }, "{detail}");
        assert_eq!(gh.lists(), if view { 0 } else { 3 }, "{detail}");
        assert_eq!(state, ("abandoned".into(), "failed".into()));
        assert!(reason.contains(detail), "{reason}");
    }
}

#[tokio::test(start_paused = true)]
async fn auth_status_abandons_without_sleep() {
    for view in [true, false] {
        let detail = if view { "HTTP 401" } else { "HTTP 403" };
        let (gh, elapsed, state, reason) = run(view, detail).await;
        assert_eq!(elapsed, Duration::ZERO, "{detail}");
        assert_eq!(gh.views(), 1, "{detail}");
        assert_eq!(gh.lists(), usize::from(!view), "{detail}");
        assert_eq!(state, ("abandoned".into(), "failed".into()));
        assert!(reason.contains(detail), "{reason}");
    }
}

#[test]
fn http_status_classifies_retries() {
    let transient = super::super::super::checks::transient_gh;
    for (detail, retry) in [
        ("HTTP 408", true),
        ("HTTP 429", true),
        ("HTTP 500", true),
        ("HTTP 599", true),
        ("HTTP 404", false),
        ("HTTP 422", false),
        ("HTTP 401", false),
        ("HTTP 403", false),
        ("HTTP 4220", true),
        ("connection reset", true),
        (
            "Post \"https://api.github.com/repos/o/r/pulls/422/reviews\": unexpected EOF",
            true,
        ),
    ] {
        assert_eq!(transient(&cmd(detail)), retry, "{detail}");
    }
    assert!(!transient(&Error::NoHome));
    let rejected = super::super::super::checks::client_rejected;
    assert!(rejected(&cmd("exit status: 1: HTTP 422")));
    assert!(!rejected(&cmd(
        "Post \"https://api.github.com/repos/o/r/pulls/422/reviews\": unexpected EOF"
    )));
}
