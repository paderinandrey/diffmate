pub use super::envelope::{finding_json, output_json, success_envelope};
pub use super::fake_call::{FakeCall, Hold};
use super::{call_numbers, mkfifo, write_executable};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_VERSION: &str = "2.1.304 (Claude Code)";
pub const PROBE_OK: &str =
    "Error: Input must be provided either through stdin or as a prompt argument when using --print";

pub struct FakeClaude {
    pub dir: PathBuf,
}

impl FakeClaude {
    pub fn install(bin: &Path, dir: &Path) -> FakeClaude {
        fs::create_dir_all(dir.join("calls")).unwrap();
        let claude = FakeClaude {
            dir: dir.to_path_buf(),
        };
        claude.set_version(DEFAULT_VERSION);
        claude.set_probe(PROBE_OK, 1);
        claude.set_run(&success_envelope(&output_json(&[], None, &[])), 0);
        let script = include_str!("fake_claude.sh").replace("@STATE@", dir.to_str().unwrap());
        write_executable(&bin.join("claude"), &script);
        claude
    }

    fn put(&self, name: &str, value: &str) {
        fs::write(self.dir.join(name), value).unwrap();
    }

    pub fn set_version(&self, line: &str) {
        self.put("version", &format!("{line}\n"));
    }

    pub fn set_probe(&self, stderr: &str, exit: i32) {
        self.put("probe_stderr", &format!("{stderr}\n"));
        self.put("probe_exit", &exit.to_string());
    }

    pub fn set_run(&self, stdout: &str, exit: i32) {
        self.put("run_stdout", stdout);
        self.put("run_exit", &exit.to_string());
    }

    pub fn hold_run(&self, nth_run: u32) -> Hold {
        let hold = Hold {
            started: self.dir.join("started"),
            release: self.dir.join("release"),
        };
        mkfifo(&hold.started);
        mkfifo(&hold.release);
        self.put("hold_run", &nth_run.to_string());
        hold
    }

    pub fn spawn_child(&self) {
        self.put("spawn_child", "");
    }

    pub fn calls(&self) -> Vec<FakeCall> {
        let calls = self.dir.join("calls");
        call_numbers(&calls)
            .into_iter()
            .map(|n| FakeCall::read(&calls, n))
            .collect()
    }

    pub fn run_calls(&self) -> Vec<FakeCall> {
        let mut calls = self.calls();
        calls.retain(|call| !call.stdin.is_empty());
        calls
    }
}
