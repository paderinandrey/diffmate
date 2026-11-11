use super::Check;
use crate::app::AppConfig;
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
    match blocking_lock(&file) {
        Ok(false) => Check::pass("runner lock: ok \u{2014} not held"),
        Ok(true) => Check::pass(held(&mut file).to_owned()),
        Err(error) => unreadable(path, &error),
    }
}

// macOS F_GETLK sets l_pid to -1 for a BSD flock. rustix debug-asserts that the pid is non-negative.
fn blocking_lock(file: &File) -> rustix::io::Result<bool> {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let probed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        fcntl_getlk(file, &Flock::from(FlockType::WriteLock))
    }));
    std::panic::set_hook(hook);
    match probed {
        Ok(result) => result.map(|blocking| blocking.is_some()),
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
