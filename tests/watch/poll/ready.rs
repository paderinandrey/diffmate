use super::super::{count, database, process, read_until, spawn_watch, with_pr};
use super::{config, search_hit, seed_tracked};
use rustix::process::{Signal, kill_process};
use std::io::{BufReader, Read};

#[test]
fn a_ready_request_becomes_one_automatic_run() {
    let sandbox = crate::support::sandbox::Sandbox::new();
    config(
        &sandbox,
        "poll_interval_seconds = 60\ndebounce_seconds = 1\n",
    );
    with_pr(&sandbox);
    sandbox.gh.route("search prs", &search_hit(false));
    let mut child = spawn_watch(&sandbox);
    let mut reader = BufReader::new(child.stderr.take().unwrap());
    let _lines = read_until(&mut reader, "succeeded (no_findings)");
    kill_process(process::pid(child.id()), Signal::TERM).unwrap();
    assert!(child.wait().unwrap().success());
    let db = database(&sandbox);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM jobs WHERE state = 'succeeded' AND origin = 'automatic'"
        ),
        1
    );
    assert_eq!(count(&db, "SELECT count(*) FROM jobs"), 1);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM targets WHERE tracking = 'tracked' AND request_seen = 1"
        ),
        1
    );
    let search = sandbox
        .gh
        .calls()
        .into_iter()
        .find(|call| call.first().is_some_and(|arg| arg == "search"))
        .unwrap();
    assert!(search.iter().any(|arg| arg == "--review-requested=@me"));
    assert!(search.iter().any(|arg| arg == "--state=open"));
    assert!(search.iter().all(|arg| arg != "--draft"));
}

#[test]
fn a_failed_search_does_not_change_request_seen() {
    let sandbox = crate::support::sandbox::Sandbox::new();
    config(
        &sandbox,
        "poll_interval_seconds = 60\ndebounce_seconds = 1\n",
    );
    seed_tracked(&sandbox);
    sandbox.gh.route_fail("search prs", "offline", 1);
    let mut child = spawn_watch(&sandbox);
    let mut reader = BufReader::new(child.stderr.take().unwrap());
    let lines = read_until(&mut reader, "poll failed:");
    assert_eq!(lines.matches("poll failed:").count(), 1);
    kill_process(process::pid(child.id()), Signal::TERM).unwrap();
    assert!(child.wait().unwrap().success());
    let mut rest = String::new();
    reader.read_to_string(&mut rest).unwrap();
    assert_eq!(rest.matches("poll failed:").count(), 0, "{lines}{rest}");
    let db = database(&sandbox);
    assert_eq!(count(&db, "SELECT request_seen FROM targets"), 1);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM targets WHERE tracking = 'tracked'"
        ),
        1
    );
    assert!(
        sandbox
            .gh
            .calls()
            .iter()
            .all(|call| call.first().is_some_and(|arg| arg != "pr"))
    );
}
