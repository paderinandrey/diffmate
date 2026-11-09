use super::*;
use std::fs;

#[test]
fn stopping_a_waiting_review_names_watch() {
    let sandbox = Sandbox::new();
    concurrency::with_two_prs(&sandbox);
    sandbox.gh.route("search prs", "[]");
    assert!(debug_enqueue(&sandbox, URL, &[]).status.success());
    let hold = sandbox.claude.hold_run(1);
    let mut watch = spawn_watch(&sandbox);
    let mut reader = BufReader::new(watch.stderr.take().unwrap());
    hold.wait_started();
    let mut review = concurrency::spawn_review(&sandbox, 2);
    let mut waiting = BufReader::new(review.stderr.take().unwrap());
    read_until(&mut waiting, "waiting:");
    kill_process(process::pid(review.id()), Signal::INT).unwrap();
    assert_eq!(review.wait().unwrap().code(), Some(1));
    read_until(
        &mut waiting,
        "o/r#2: Stopped waiting; the job stays queued and watch will run it",
    );
    hold.release();
    read_until(&mut reader, "job 2 review o/r#2 succeeded");
    kill_process(process::pid(watch.id()), Signal::TERM).unwrap();
    assert!(watch.wait().unwrap().success());
}

#[test]
fn review_while_watch_runs_prints_the_result() {
    let sandbox = Sandbox::new();
    concurrency::with_two_prs(&sandbox);
    sandbox.gh.route("search prs", "[]");
    assert!(debug_enqueue(&sandbox, URL, &[]).status.success());
    let mut watch = spawn_watch(&sandbox);
    let mut reader = BufReader::new(watch.stderr.take().unwrap());
    let mut watch_err = read_until(&mut reader, "job 1 review o/r#1 succeeded");
    let output = sandbox.run(&["review", &concurrency::url(2)]);
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("o/r#2 at ") && stdout.contains(": succeeded"),
        "{stdout}"
    );
    watch_err.push_str(&read_until(&mut reader, "job 2 review o/r#2 succeeded"));
    assert!(
        watch_err.contains("job 2 review o/r#2 succeeded"),
        "{watch_err}"
    );
    let past_shared = stdout.replacen("succeeded", "", 1);
    assert!(
        !watch_err.contains(past_shared.trim()),
        "watch stderr contains the review report past succeeded\n{watch_err}\n{stdout}"
    );
    assert!(!watch_err.lines().any(finding_title_line), "{watch_err}");
    assert_eq!(runs_for(&sandbox, &concurrency::url(2)), 1);
    kill_process(process::pid(watch.id()), Signal::TERM).unwrap();
    assert!(watch.wait().unwrap().success());
}

fn finding_title_line(line: &str) -> bool {
    let text = line.split_once("Z ").map_or(line, |(_, rest)| rest).trim();
    text.split_once(" [")
        .and_then(|(_, rest)| rest.split_once("] "))
        .is_some_and(|(severity, _)| matches!(severity, "high" | "medium" | "low"))
}

fn runs_for(sandbox: &Sandbox, url: &str) -> usize {
    sandbox
        .claude
        .run_calls()
        .into_iter()
        .filter(|call| {
            call.context.as_ref().is_some_and(|dir| {
                fs::read_to_string(dir.join("pr.md")).is_ok_and(|text| text.contains(url))
            })
        })
        .count()
}
