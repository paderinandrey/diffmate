use crate::support::sandbox::Sandbox;
use crate::watch_helpers::read_until;
use rustix::process::{Pid, Signal, kill_process};
use std::io::BufReader;
use std::process::{Child, ChildStderr, Command, Stdio};

pub struct HeldChild(pub Child);

impl Drop for HeldChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

pub fn spawn_at(command: &mut Command, point: &str) -> (HeldChild, BufReader<ChildStderr>) {
    let mut child = command
        .env("DIFFMATE_CRASH_POINT", point)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let reader = BufReader::new(child.stderr.take().unwrap());
    (HeldChild(child), reader)
}

pub fn kill(child: &mut Child) {
    child.kill().unwrap();
    assert!(!child.wait().unwrap().success());
}

pub fn signal(child: &Child, signal: Signal) {
    let pid = Pid::from_raw(child.id().try_into().unwrap()).unwrap();
    kill_process(pid, signal).unwrap();
}

pub fn stop(child: &mut Child) {
    signal(child, Signal::TERM);
    assert!(child.wait().unwrap().success());
}

pub fn crash_after_stop(sandbox: &Sandbox, command: &mut Command, stop: Signal) {
    let hold = sandbox.claude.hold_run(1);
    let (mut child, mut reader) = spawn_at(command, "stop-stored");
    hold.wait_started();
    signal(&child.0, stop);
    read_until(&mut reader, "crash point reached: stop-stored");
    kill(&mut child.0);
}
