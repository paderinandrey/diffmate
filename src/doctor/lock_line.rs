use super::Check;
use crate::app::AppConfig;
#[cfg(not(target_os = "linux"))]
use rustix::process::{Flock, FlockType, fcntl_getlk};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub(super) async fn check(config: &AppConfig) -> Check {
    let path = config.paths.lock();
    let inspected = tokio::task::spawn_blocking(move || inspect(&path)).await;
    match inspected {
        Ok(check) => check,
        Err(error) => Check::fail(format!("runner lock: {error}")),
    }
}

fn inspect(path: &Path) -> Check {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return missing(path),
        Err(error) => return unreadable(path, &error),
    };
    match held_exclusively(&file) {
        Ok(false) => Check::pass("runner lock: ok \u{2014} not held"),
        Ok(true) => Check::pass(held(&mut file).to_owned()),
        Err(Probe::File(error)) => unreadable(path, &error),
        Err(Probe::Proc(error)) => {
            Check::fail(format!("runner lock: cannot read /proc/locks: {error}"))
        }
    }
}

enum Probe {
    File(io::Error),
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    Proc(io::Error),
}

fn held_exclusively(file: &File) -> Result<bool, Probe> {
    // Linux `flock` is invisible to F_GETLK. `/proc/locks` lists it without taking it.
    #[cfg(target_os = "linux")]
    {
        linux_flock(file)
    }
    #[cfg(not(target_os = "linux"))]
    {
        blocking_lock(file).map_err(Probe::File)
    }
}

#[cfg(target_os = "linux")]
fn linux_flock(file: &File) -> Result<bool, Probe> {
    use std::os::unix::fs::MetadataExt;
    let meta = file.metadata().map_err(Probe::File)?;
    let text = std::fs::read_to_string("/proc/locks").map_err(Probe::Proc)?;
    Ok(proc_locks_held(&text, meta.dev(), meta.ino()))
}

#[cfg(not(target_os = "linux"))]
fn blocking_lock(file: &File) -> io::Result<bool> {
    // macOS F_GETLK sets l_pid to -1 for a BSD flock. rustix debug-asserts that the pid is non-negative.
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let probed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        fcntl_getlk(file, &Flock::from(FlockType::WriteLock))
    }));
    std::panic::set_hook(hook);
    match probed {
        Ok(result) => result
            .map(|blocking| blocking.is_some())
            .map_err(io::Error::from),
        Err(_) => Ok(true),
    }
}

fn missing(path: &Path) -> Check {
    let path = path.display();
    Check::fail(format!(
        "runner lock: {path} is missing; run diffmate watch or diffmate review once; doctor does not create the file"
    ))
}

fn unreadable(path: &Path, error: &impl std::fmt::Display) -> Check {
    Check::fail(format!(
        "runner lock: {}: {error}; chmod 600",
        path.display()
    ))
}

fn held(file: &mut File) -> &'static str {
    let mut bytes = [0_u8; 16];
    let read = file.read(&mut bytes).unwrap_or(0);
    let text = String::from_utf8_lossy(&bytes[..read]);
    match text.lines().next().unwrap_or("") {
        "watch" => "runner lock: ok \u{2014} held by watch",
        "review" => "runner lock: ok \u{2014} held by review",
        "tui" => "runner lock: ok \u{2014} held by tui",
        _ => "runner lock: ok \u{2014} held, role unreadable",
    }
}

#[cfg(any(test, target_os = "linux"))]
pub(super) fn proc_locks_held(text: &str, dev: u64, ino: u64) -> bool {
    let major = linux_major(dev);
    let minor = linux_minor(dev);
    text.lines()
        .any(|line| flock_write(line, major, minor, ino))
}

#[cfg(any(test, target_os = "linux"))]
fn flock_write(line: &str, major: u32, minor: u32, ino: u64) -> bool {
    let mut parts = line.split_whitespace();
    if parts.next().is_none() {
        return false;
    }
    // A blocked waiter is `N: -> FLOCK ...` and does not hold the lock.
    if parts.next() != Some("FLOCK")
        || parts.next() != Some("ADVISORY")
        || parts.next() != Some("WRITE")
        || parts.next().is_none()
    {
        return false;
    }
    let Some((seen_major, seen_minor, seen_ino)) = parts.next().and_then(parse_loc) else {
        return false;
    };
    seen_major == major && seen_minor == minor && seen_ino == ino
}

#[cfg(any(test, target_os = "linux"))]
fn parse_loc(loc: &str) -> Option<(u32, u32, u64)> {
    let mut parts = loc.split(':');
    let major = u32::from_str_radix(parts.next()?, 16).ok()?;
    let minor = u32::from_str_radix(parts.next()?, 16).ok()?;
    let ino = parts.next()?.parse().ok()?;
    Some((major, minor, ino))
}

// glibc `major`/`minor`, the split the kernel uses when it prints `/proc/locks`.
#[cfg(any(test, target_os = "linux"))]
pub(super) fn linux_major(dev: u64) -> u32 {
    let value = ((dev >> 32) & 0xffff_f000) | ((dev >> 8) & 0xfff);
    u32::try_from(value).unwrap_or(0)
}

#[cfg(any(test, target_os = "linux"))]
pub(super) fn linux_minor(dev: u64) -> u32 {
    let value = ((dev >> 12) & 0xffff_ff00) | (dev & 0xff);
    u32::try_from(value).unwrap_or(0)
}
