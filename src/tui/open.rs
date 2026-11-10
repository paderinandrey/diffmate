use std::io;
use std::process::Stdio;
use tokio::process::Command;

pub(crate) const PROGRAM: &str = "open";

pub(crate) async fn open_pr(program: &str, url: &str) -> io::Result<()> {
    Command::new(program)
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?
        .wait()
        .await
        .map(drop)
}
