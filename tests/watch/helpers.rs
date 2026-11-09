use crate::support::sandbox::Sandbox;
use std::io::BufRead;
use std::process::{Child, Output, Stdio};

pub fn spawn_watch(sandbox: &Sandbox) -> Child {
    sandbox
        .diffmate()
        .arg("watch")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

pub fn debug_enqueue(sandbox: &Sandbox, url: &str, flags: &[&str]) -> Output {
    sandbox
        .diffmate()
        .args(["debug", "enqueue", url])
        .args(flags)
        .output()
        .unwrap()
}

pub fn read_until(reader: &mut impl BufRead, needle: &str) -> String {
    let mut text = String::new();
    loop {
        let mut line = String::new();
        assert_ne!(
            reader.read_line(&mut line).unwrap(),
            0,
            "missing {needle}: {text}"
        );
        text.push_str(&line);
        if line.contains(needle) {
            return text;
        }
    }
}
