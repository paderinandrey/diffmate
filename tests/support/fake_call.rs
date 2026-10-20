use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

pub struct FakeCall {
    pub n: u32,
    pub argv: Vec<String>,
    pub env: Vec<(String, String)>,
    pub cwd: PathBuf,
    pub stdin: Vec<u8>,
    pub context: Option<PathBuf>,
    pub pid: Option<u32>,
    pub child_pid: Option<u32>,
}

impl FakeCall {
    pub fn read(calls: &Path, n: u32) -> FakeCall {
        let dir = calls.join(n.to_string());
        let text = |name: &str| fs::read_to_string(dir.join(name)).unwrap_or_default();
        let number = |name: &str| text(name).trim().parse().ok();
        FakeCall {
            n,
            argv: text("argv").lines().map(String::from).collect(),
            env: text("env")
                .lines()
                .filter_map(|line| line.split_once('='))
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
            cwd: PathBuf::from(text("cwd").trim()),
            stdin: fs::read(dir.join("stdin")).unwrap_or_default(),
            context: dir.join("context").exists().then(|| dir.join("context")),
            pid: number("pid"),
            child_pid: number("child.pid"),
        }
    }
}

pub struct Hold {
    pub started: PathBuf,
    pub release: PathBuf,
}

impl Hold {
    pub fn wait_started(&self) {
        let mut line = String::new();
        let file = File::open(&self.started).unwrap();
        BufReader::new(file).read_line(&mut line).unwrap();
    }

    pub fn release(&self) {
        let mut file = OpenOptions::new().write(true).open(&self.release).unwrap();
        writeln!(file, "go").unwrap();
    }
}
