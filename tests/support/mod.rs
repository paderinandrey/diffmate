#![allow(dead_code, unused_imports)]

mod envelope;
mod fake_call;
pub mod fake_claude;
pub mod fake_gh;
pub mod poison;
pub mod sandbox;
pub mod upstream;

use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use upstream::real_git;

pub fn write_executable(path: &Path, script: &str) {
    let mut file = File::create(path).unwrap();
    file.write_all(script.as_bytes()).unwrap();
    file.sync_all().unwrap();
    drop(file);
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

pub fn mkfifo(path: &Path) {
    let status = Command::new("mkfifo").arg(path).status().unwrap();
    assert!(status.success());
}

pub fn call_numbers(calls: &Path) -> Vec<u32> {
    let mut numbers: Vec<u32> = fs::read_dir(calls)
        .unwrap()
        .map(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_str()
                .unwrap()
                .parse()
                .unwrap()
        })
        .collect();
    numbers.sort_unstable();
    numbers
}

pub fn exec(program: &Path, args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(stdin.as_bytes());
    child.wait_with_output().unwrap()
}

pub fn signal(flag: &str, pid: u32) -> bool {
    let status = Command::new("kill").args([flag, &pid.to_string()]).status();
    status.unwrap().success()
}

pub fn git_in(dir: &Path, home: &Path, args: &[&str]) -> String {
    let path = std::env::var_os("PATH").unwrap();
    let mut command = Command::new(real_git());
    command.args(args).current_dir(dir).env_clear();
    command
        .env("PATH", path)
        .env("HOME", home)
        .env("LC_ALL", "C");
    command
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1");
    for who in ["AUTHOR", "COMMITTER"] {
        command.env(format!("GIT_{who}_NAME"), "Fixture");
        command.env(format!("GIT_{who}_EMAIL"), "fixture@example.com");
        command.env(format!("GIT_{who}_DATE"), "2026-01-01T00:00:00Z");
    }
    let output = command.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "git {args:?}: {stderr}");
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}
