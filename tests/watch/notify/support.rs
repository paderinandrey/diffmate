use super::super::super::{count, database, process, read_until};
use crate::support::sandbox::Sandbox;
use crate::support::write_executable;
use rustix::process::{Signal, kill_process};
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};

pub struct Osascript {
    pub log: PathBuf,
    pub status: PathBuf,
}

pub fn install(sandbox: &Sandbox) -> Osascript {
    let log = sandbox.root.join("osascript.log");
    let status = sandbox.root.join("osascript.status");
    let script = format!(
        r#"#!/bin/sh
log={log}
status={status}
printf '%s\n' "$0" "$@" >> "$log"
printf 'LC_ALL=%s\n' "$LC_ALL" >> "$log"
printf 'NOT_A_TOOL_VAR=%s\n' "$NOT_A_TOOL_VAR" >> "$log"
printf '%s\n' '---' >> "$log"
if [ -f "$status" ]; then
  exit "$(cat "$status")"
fi
exit 0
"#,
        log = quote(&log),
        status = quote(&status),
    );
    write_executable(&sandbox.bin.join("osascript"), &script);
    fs::write(&log, "").unwrap();
    Osascript { log, status }
}

fn quote(path: &Path) -> String {
    format!("'{}'", path.display())
}

pub fn spawn(sandbox: &Sandbox) -> Child {
    sandbox
        .diffmate()
        .arg("watch")
        .env("NOT_A_TOOL_VAR", "secret")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

pub fn finish(child: &mut Child, needle: &str) -> String {
    let mut reader = BufReader::new(child.stderr.take().unwrap());
    let head = read_until(&mut reader, needle);
    kill_process(process::pid(child.id()), Signal::TERM).unwrap();
    let tail = read_until(&mut reader, "watch stopped");
    let status = child.wait().unwrap();
    assert!(status.success(), "{status}");
    let mut text = head;
    text.push_str(&tail);
    text
}

pub fn text(script: &Osascript) -> String {
    fs::read_to_string(&script.log).unwrap()
}

pub fn scripts(log: &str) -> Vec<&str> {
    log.lines()
        .filter(|line| line.starts_with("display notification "))
        .collect()
}

pub fn count_sql(sandbox: &Sandbox, sql: &str) -> i64 {
    count(&database(sandbox), sql)
}
