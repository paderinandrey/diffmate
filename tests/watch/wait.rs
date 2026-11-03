use super::*;

#[test]
fn watch_waits_for_the_lock_and_stops_on_sigterm() {
    for signal in [Signal::TERM, Signal::INT] {
        let sandbox = Sandbox::new();
        with_pr(&sandbox);
        let hold = sandbox.claude.hold_run(1);
        let review = concurrency::spawn_review(&sandbox, 1);
        hold.wait_started();
        let mut watch = spawn_watch(&sandbox);
        let mut reader = BufReader::new(watch.stderr.take().unwrap());
        read_until(&mut reader, "waiting for the runner lock");
        assert!(watch.try_wait().unwrap().is_none());
        kill_process(process::pid(watch.id()), signal).unwrap();
        assert!(watch.wait().unwrap().success());
        let mut remaining = String::new();
        reader.read_to_string(&mut remaining).unwrap();
        assert!(remaining.contains("watch stopped (shutdown)"));
        assert_eq!(count(&database(&sandbox), "SELECT count(*) FROM jobs"), 1);
        hold.release();
        assert!(review.wait_with_output().unwrap().status.success());
    }
}
