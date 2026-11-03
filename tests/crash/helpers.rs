use crate::support::{git_in, sandbox::Sandbox};
use crate::tracer::{count, database};
use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

pub fn crash_at(command: &mut Command, point: &str) {
    crash_at_ready(command, point, || {});
}

pub fn crash_at_ready(command: &mut Command, point: &str, ready: impl FnOnce()) {
    let mut child = command
        .env("DIFFMATE_CRASH_POINT", point)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut reader = BufReader::new(child.stderr.take().unwrap());
    let needle = format!("crash point reached: {point}");
    let mut output = String::new();
    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line).unwrap();
        if read == 0 {
            let status = child.wait().unwrap();
            panic!("missing {needle}; exited {status}: {output}");
        }
        output.push_str(&line);
        if line.trim() == needle {
            break;
        }
    }
    ready();
    child.kill().unwrap();
    assert!(!child.wait().unwrap().success());
}

pub fn assert_clean(sandbox: &Sandbox) {
    let db = database(sandbox);
    for table in ["runs", "jobs"] {
        assert_eq!(
            count(
                &db,
                &format!("SELECT count(*) FROM {table} WHERE state = 'running'")
            ),
            0
        );
    }
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM (SELECT job_id FROM runs JOIN jobs ON jobs.id = runs.job_id WHERE jobs.origin = 'manual' GROUP BY job_id HAVING count(*) > 1)"
        ),
        0
    );
    let runs = sandbox.cache_dir().join("runs");
    if runs.exists() {
        assert_eq!(fs::read_dir(runs).unwrap().count(), 0);
    }
    for owner in fs::read_dir(sandbox.state_dir().join("repos")).unwrap() {
        for repo in fs::read_dir(owner.unwrap().path()).unwrap() {
            let path = repo.unwrap().path();
            let listed = git_in(&path, &sandbox.home, &["worktree", "list", "--porcelain"]);
            assert_eq!(listed.matches("worktree ").count(), 1, "{listed}");
            assert!(listed.lines().any(|line| line == "bare"), "{listed}");
        }
    }
}
