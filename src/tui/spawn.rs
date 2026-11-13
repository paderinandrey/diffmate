use std::future::Future;
use std::io;
use std::path::Path;
use std::pin::Pin;
use std::process::{ExitStatus, Stdio};
use tokio::process::Command;

pub(super) type ExitFuture<'a> = Pin<Box<dyn Future<Output = io::Result<ExitStatus>> + Send + 'a>>;
pub(super) type OpenFuture<'a> = Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>>;

pub(super) trait Editor: Send {
    fn launch<'a>(&'a self, argv: &'a [String], path: &'a Path) -> ExitFuture<'a>;
}

pub(super) trait Browser: Send {
    fn launch<'a>(&'a self, program: &'a str, url: &'a str) -> OpenFuture<'a>;
}

pub(super) struct EditorCommand;

impl Editor for EditorCommand {
    fn launch<'a>(&'a self, argv: &'a [String], path: &'a Path) -> ExitFuture<'a> {
        let _ = self;
        Box::pin(async move {
            let Some((program, rest)) = argv.split_first() else {
                return Err(io::Error::other("editor"));
            };
            Command::new(program)
                .args(rest)
                .arg(path)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?
                .wait()
                .await
        })
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
