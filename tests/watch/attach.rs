use super::*;

#[test]
fn stopping_a_waiting_review_names_watch() {
    let sandbox = Sandbox::new();
    concurrency::with_two_prs(&sandbox);
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
    assert!(debug_enqueue(&sandbox, URL, &[]).status.success());
    let mut watch = spawn_watch(&sandbox);
    let mut reader = BufReader::new(watch.stderr.take().unwrap());
    read_until(&mut reader, "job 1 review o/r#1 succeeded");
    let output = sandbox.run(&["review", &concurrency::url(2)]);
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("o/r#2 at ") && stdout.contains(": succeeded"),
        "{stdout}"
    );
    read_until(&mut reader, "job 2 review o/r#2 succeeded");
    kill_process(process::pid(watch.id()), Signal::TERM).unwrap();
    assert!(watch.wait().unwrap().success());
}
