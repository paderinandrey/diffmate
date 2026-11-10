use super::RunnerLock;
use crate::error::Error;
use std::io::{Read, Seek, Write};
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Role {
    Review,
    Watch,
    Tui,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Review => "review",
            Self::Watch => "watch",
            Self::Tui => "tui",
        }
    }
}

impl RunnerLock {
    pub async fn announce(&self, role: Role) -> Result<(), Error> {
        let file = Arc::clone(&self.file);
        tokio::task::spawn_blocking(move || {
            let write = || -> std::io::Result<()> {
                let mut file = file.try_clone()?;
                file.set_len(0)?;
                file.rewind()?;
                writeln!(file, "{}", role.as_str())
            };
            write().map_err(Error::io("announce the runner role"))
        })
        .await?
    }
}

pub(crate) async fn holder(path: &Path) -> Option<Role> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .ok()?;
        if !matches!(file.try_lock(), Err(std::fs::TryLockError::WouldBlock)) {
            return None;
        }
        let mut text = String::new();
        file.read_to_string(&mut text).ok()?;
        match text.lines().next()? {
            "review" => Some(Role::Review),
            "watch" => Some(Role::Watch),
            "tui" => Some(Role::Tui),
            _ => None,
        }
    })
    .await
    .ok()
    .flatten()
}
