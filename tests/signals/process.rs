use rustix::process::Pid;
use std::process::Command;
use std::time::{Duration, Instant};

pub(super) fn pid(raw: u32) -> Pid {
    Pid::from_raw(i32::try_from(raw).unwrap()).unwrap()
}

pub(super) fn running(raw: u32) -> bool {
    let output = Command::new("ps")
        .args(["-o", "stat=", "-p", &raw.to_string()])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.trim().is_empty(), "ps failed: {stderr}");
    assert!(output.status.success() || output.status.code() == Some(1));
    let stat = String::from_utf8_lossy(&output.stdout);
    !stat.trim().is_empty() && !stat.trim().starts_with('Z')
}

pub(super) fn wait_until_gone(raw: u32) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while running(raw) {
        assert!(Instant::now() < deadline, "process {raw} did not stop");
        std::thread::yield_now();
    }
}
