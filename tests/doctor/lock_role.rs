use super::support::sandbox::Sandbox;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::PathBuf;
use std::process::ExitStatus;

fn sandbox() -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.gh.route("auth status", "logged in\n");
    for dir in [sandbox.state_dir(), sandbox.cache_dir()] {
        fs::create_dir_all(&dir).unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    }
    sandbox
}

fn hold(sandbox: &Sandbox, bytes: &[u8]) -> (PathBuf, File) {
    let path = sandbox.state_dir().join("runner.lock");
    let mut lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&path)
        .unwrap();
    lock.write_all(bytes).unwrap();
    lock.try_lock().unwrap();
    (path, lock)
}

fn lock_line(sandbox: &Sandbox) -> (ExitStatus, String) {
    let output = sandbox.run(&["doctor"]);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 5, "{stdout}");
    (output.status, stdout.lines().nth(3).unwrap().to_owned())
}

#[test]
fn held_review_tui_and_unreadable_role_keep_the_flock() {
    let cases: [(&[u8], &str); 4] = [
        (b"review\n", "runner lock: ok \u{2014} held by review"),
        (b"tui\n", "runner lock: ok \u{2014} held by tui"),
        (b"", "runner lock: ok \u{2014} held, role unreadable"),
        (b"nope\n", "runner lock: ok \u{2014} held, role unreadable"),
    ];
    for (bytes, expect) in cases {
        let sandbox = sandbox();
        let (path, _lock) = hold(&sandbox, bytes);
        let (status, line) = lock_line(&sandbox);
        assert!(status.success(), "{line}");
        assert_eq!(line, expect);
        let probe = File::open(&path).unwrap();
        assert!(matches!(probe.try_lock(), Err(TryLockError::WouldBlock)));
    }
}

#[test]
fn unreadable_lock_names_chmod_and_keeps_mode_000() {
    let sandbox = sandbox();
    let path = sandbox.state_dir().join("runner.lock");
    fs::write(&path, b"watch\n").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    let entries = fs::read_dir(sandbox.state_dir()).unwrap().count();
    let (status, line) = lock_line(&sandbox);
    assert_eq!(status.code(), Some(1), "{line}");
    assert!(line.contains(&path.display().to_string()), "{line}");
    assert!(line.contains("chmod 600"), "{line}");
    assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0);
    assert_eq!(fs::read_dir(sandbox.state_dir()).unwrap().count(), entries);
}
