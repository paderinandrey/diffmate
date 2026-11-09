use super::super::{process, spawn_watch};
use crate::support::sandbox::Sandbox;
use rustix::process::{Signal, kill_process};
use std::io::{BufRead, BufReader};
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct Running {
    child: Child,
    err: Arc<Mutex<String>>,
}

impl Running {
    pub fn spawn(sandbox: &Sandbox) -> Self {
        let mut child = spawn_watch(sandbox);
        let err = Arc::new(Mutex::new(String::new()));
        let sink = Arc::clone(&err);
        let stderr = child.stderr.take().unwrap();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                sink.lock().unwrap().push_str(&line);
                line.clear();
            }
        });
        Self { child, err }
    }

    pub fn wait_until(&mut self, ready: impl FnMut(&str) -> bool) {
        let mut ready = ready;
        loop {
            let err = self.err.lock().unwrap().clone();
            assert!(!err.contains("error:"), "{err}");
            assert!(!err.contains("poll failed:"), "{err}");
            match self.child.try_wait().unwrap() {
                Some(status) => panic!("watch exited {status}: {err}"),
                None if ready(&err) => return,
                None => std::thread::yield_now(),
            }
        }
    }

    pub fn stop(mut self) {
        kill_process(process::pid(self.child.id()), Signal::TERM).unwrap();
        let status = self.child.wait().unwrap();
        assert!(status.success(), "{status}: {}", self.err.lock().unwrap());
    }
}

pub fn query(sandbox: &Sandbox, sql: &str) -> i64 {
    if !sandbox.db_path().exists() {
        return 0;
    }
    let db = rusqlite::Connection::open_with_flags(
        sandbox.db_path(),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    db.busy_timeout(Duration::from_secs(5)).unwrap();
    match db.query_row(sql, [], |row| row.get(0)) {
        Ok(value) => value,
        Err(rusqlite::Error::QueryReturnedNoRows) => 0,
        Err(error) if error.to_string().contains("no such table") => 0,
        Err(error) => panic!("{error}"),
    }
}

pub fn calls(sandbox: &Sandbox, command: &str) -> usize {
    sandbox
        .gh
        .calls()
        .iter()
        .filter(|call| call.first().is_some_and(|arg| arg == command))
        .count()
}

pub fn view(head: &str, state: &str, draft: bool) -> String {
    format!(
        "{{\"number\":1,\"url\":\"https://github.com/o/r/pull/1\",\"title\":\"t\",\"body\":\"b\",\"author\":{{\"login\":\"a\"}},\"baseRefName\":\"main\",\"headRefName\":\"feature\",\"headRefOid\":\"{head}\",\"isCrossRepository\":false,\"state\":\"{state}\",\"isDraft\":{draft}}}"
    )
}
