use std::future::Future;
use std::io;
use std::path::Path;
use std::pin::Pin;
use std::process::{ExitStatus, Stdio};
use std::sync::{Arc, Mutex, PoisonError};
use tokio::process::{Child, Command};
use tokio::sync::oneshot;

pub(super) type ExitFuture<'a> = Pin<Box<dyn Future<Output = io::Result<ExitStatus>> + Send + 'a>>;
pub(super) type OpenFuture<'a> = Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>>;

pub(super) trait Editor: Send {
    fn launch<'a>(&'a self, argv: &'a [String], path: &'a Path) -> ExitFuture<'a>;
}

pub(super) trait Browser: Send {
    fn launch<'a>(&'a self, program: &'a str, url: &'a str) -> OpenFuture<'a>;
}

pub(super) struct ReapGate {
    pending: Mutex<Vec<oneshot::Receiver<()>>>,
}

impl ReapGate {
    pub(super) fn new() -> Self {
        Self {
            pending: Mutex::new(Vec::new()),
        }
    }

    fn track(&self) -> oneshot::Sender<()> {
        let (tx, rx) = oneshot::channel();
        self.pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(rx);
        tx
    }

    pub(super) async fn drain(&self) {
        let waiting = {
            let mut guard = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
            std::mem::take(&mut *guard)
        };
        for waiter in waiting {
            let _ = waiter.await;
        }
    }
}

pub(super) struct EditorCommand {
    reap: Arc<ReapGate>,
}

impl EditorCommand {
    pub(super) fn new(reap: Arc<ReapGate>) -> Self {
        Self { reap }
    }

    #[cfg(test)]
    pub(super) async fn wait_reaped(&self) {
        self.reap.drain().await;
    }
}

impl Editor for EditorCommand {
    fn launch<'a>(&'a self, argv: &'a [String], path: &'a Path) -> ExitFuture<'a> {
        let reap = Arc::clone(&self.reap);
        Box::pin(async move {
            let Some((program, rest)) = argv.split_first() else {
                return Err(io::Error::other("editor"));
            };
            let child = Command::new(program)
                .args(rest)
                .arg(path)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .kill_on_drop(true)
                .spawn()?;
            EditorProc {
                child: Some(child),
                reap,
            }
            .wait()
            .await
        })
    }
}

struct EditorProc {
    child: Option<Child>,
    reap: Arc<ReapGate>,
}

impl EditorProc {
    async fn wait(&mut self) -> io::Result<ExitStatus> {
        let status = {
            let Some(child) = self.child.as_mut() else {
                return Err(io::Error::other("editor"));
            };
            child.wait().await?
        };
        self.child = None;
        Ok(status)
    }
}

impl Drop for EditorProc {
    fn drop(&mut self) {
        let Some(mut child) = self.child.take() else {
            return;
        };
        // Tokio Child has no blocking wait, and Drop cannot await. The reaper
        // thread signals `ReapGate`; shutdown awaits that gate before restoring
        // the terminal.
        let done = self.reap.track();
        std::thread::spawn(move || {
            if child.start_kill().is_ok() {
                while matches!(child.try_wait(), Ok(None)) {
                    std::thread::yield_now();
                }
            } else {
                let _ = child.try_wait();
            }
            let _ = done.send(());
        });
    }
}

pub(super) struct BrowserCommand;

impl Browser for BrowserCommand {
    fn launch<'a>(&'a self, program: &'a str, url: &'a str) -> OpenFuture<'a> {
        let _ = self;
        let program = program.to_owned();
        let url = url.to_owned();
        Box::pin(async move {
            Command::new(program)
                .arg(url)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?
                .wait()
                .await
                .map(drop)
        })
    }
}
