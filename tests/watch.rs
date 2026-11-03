#[allow(dead_code)]
#[path = "concurrency/helpers.rs"]
mod concurrency;
#[path = "watch/debug.rs"]
mod debug;
#[path = "watch/helpers.rs"]
mod helpers;
#[path = "watch/persistence.rs"]
mod persistence;
#[allow(dead_code)]
#[path = "signals/process.rs"]
mod process;
#[path = "watch/shutdown.rs"]
mod shutdown;
mod support;
#[allow(dead_code)]
mod tracer;
#[path = "watch/wait.rs"]
mod wait;

use helpers::{debug_enqueue, read_until, spawn_watch};
use rustix::process::{Signal, kill_process};
use std::io::{BufReader, Read};
use support::sandbox::Sandbox;
use tracer::{URL, count, database, with_pr};

#[test]
fn watch_runs_an_automatic_job_and_stops_when_idle() {
    let sandbox = Sandbox::new();
    with_pr(&sandbox);
    let queued = debug_enqueue(&sandbox, URL, &["--automatic"]);
    assert!(queued.status.success(), "{queued:?}");
    assert_eq!(String::from_utf8_lossy(&queued.stdout), "job 1 queued\n");
    let mut child = spawn_watch(&sandbox);
    let mut reader = BufReader::new(child.stderr.take().unwrap());
    let lines = read_until(&mut reader, "succeeded (no_findings)");
    assert!(lines.contains("job 1 review o/r#1 started\n"), "{lines}");
    for line in lines.lines() {
        assert_eq!(&line[4..5], "-");
        assert_eq!(&line[19..21], "Z ");
    }
    kill_process(process::pid(child.id()), Signal::TERM).unwrap();
    assert!(child.wait().unwrap().success());
    let mut stopped = String::new();
    reader.read_to_string(&mut stopped).unwrap();
    assert!(stopped.contains("watch stopped (shutdown)"));
    assert_eq!(
        count(
            &database(&sandbox),
            "SELECT count(*) FROM jobs WHERE state = 'succeeded' AND origin = 'automatic'"
        ),
        1
    );
}

#[test]
fn debug_is_hidden_from_help() {
    let output = Sandbox::new().run(&["--help"]);
    let help = String::from_utf8_lossy(&output.stdout);
    for command in ["review", "status", "watch"] {
        assert!(help.contains(command), "{help}");
    }
    assert!(!help.contains("debug"));
}
