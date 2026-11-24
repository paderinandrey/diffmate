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
        .confirm_publication(
            store.displayed_draft(1).await,
            "COMMENT",
            RUN,
            &super::super::support::process(),
        )
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
async fn each_check_has_a_per_attempt_deadline_and_never_posts_on_exhaustion() {
    use super::boom::Check;
    for (check, label, index) in [
        (Check::Head, "HEAD check", 0),
        (Check::Login, "login check", 1),
        (Check::Reviews, "list reviews", 2),
    ] {
        for (budget, seconds) in [(0, 5), (2, 21)] {
            let (_tmp, store, _, _) = open_gh(RUN).await;
            let gh = Boom::stalling(check, usize::MAX);
            let job = queued(&store).await;
            let start = tokio::time::Instant::now();
            crate::publish::execute_publish(&store, &gh, job, budget)
                .await
                .unwrap();
            assert_eq!(start.elapsed(), Duration::from_secs(seconds));
            let mut expected = [0; 4];
            expected[..index].fill(1);
            expected[index] = usize::try_from(budget).unwrap() + 1;
            assert_eq!(gh.calls(), expected);
            assert_eq!(pair(&store).await, ("abandoned".into(), "failed".into()));
            let reason: String =
                super::super::support::cell(&store, "SELECT reason FROM publications").await;
            assert_eq!(reason, format!("{label} timed out"));
        }
    }
}

#[tokio::test(start_paused = true)]
async fn each_check_can_recover_after_its_first_timeout_and_post_once() {
    use super::boom::Check;
    for (index, check) in [Check::Head, Check::Login, Check::Reviews]
        .into_iter()
        .enumerate()
    {
        let (_tmp, store, _, _) = open_gh(RUN).await;
        let gh = Boom::stalling(check, 1);
        let job = queued(&store).await;
        let start = tokio::time::Instant::now();
        crate::publish::execute_publish(&store, &gh, job, 1)
            .await
            .unwrap();
        assert_eq!(start.elapsed(), Duration::from_secs(6));
        let mut expected = [1; 4];
        expected[index] = 2;
        assert_eq!(gh.calls(), expected);
        assert_eq!(pair(&store).await, ("submitted".into(), "succeeded".into()));
    }
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
}
