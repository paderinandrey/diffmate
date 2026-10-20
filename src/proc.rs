use crate::error::Error;
use std::io;
use std::process::{ExitStatus, Stdio};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, ChildStdin, Command};

pub(crate) struct Limits {
    pub stdout: usize,
    pub stderr: usize,
}

pub(crate) struct Captured {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub stdout_overflow: bool,
}

async fn feed(pipe: Option<ChildStdin>, data: Option<Vec<u8>>) {
    if let (Some(mut pipe), Some(data)) = (pipe, data) {
        let _ = pipe.write_all(&data).await;
    }
}

async fn drain<R: AsyncRead + Unpin>(pipe: Option<R>, limit: usize) -> io::Result<(Vec<u8>, bool)> {
    let Some(mut pipe) = pipe else {
        return Ok((Vec::new(), false));
    };
    let (mut kept, mut overflow, mut chunk) = (Vec::new(), false, vec![0_u8; 8192]);
    loop {
        let read = pipe.read(&mut chunk).await?;
        if read == 0 {
            return Ok((kept, overflow));
        }
        let room = limit.saturating_sub(kept.len());
        kept.extend_from_slice(&chunk[..read.min(room)]);
        overflow |= read > room;
    }
}

pub(crate) async fn finish(
    mut child: Child,
    stdin: Option<Vec<u8>>,
    limits: Limits,
) -> io::Result<Captured> {
    let (out, err) = (child.stdout.take(), child.stderr.take());
    let input = feed(child.stdin.take(), stdin);
    let ((), out, err, status) = tokio::join!(
        input,
        drain(out, limits.stdout),
        drain(err, limits.stderr),
        child.wait()
    );
    let ((stdout, stdout_overflow), (stderr, _)) = (out?, err?);
    let status = status?;
    Ok(Captured {
        status,
        stdout,
        stderr,
        stdout_overflow,
    })
}

pub(crate) fn excerpt(bytes: &[u8], max: usize) -> String {
    String::from_utf8_lossy(&bytes[..bytes.len().min(max)]).into_owned()
}

pub(crate) async fn run_tool(
    program: &'static str,
    mut command: Command,
) -> Result<Vec<u8>, Error> {
    command.stdin(Stdio::null()).stdout(Stdio::piped());
    command.stderr(Stdio::piped()).kill_on_drop(true);
    let words: Vec<_> = command.as_std().get_args().collect();
    let words: Vec<_> = words.iter().map(|word| word.to_string_lossy()).collect();
    let args = words.join(" ");
    let child = command
        .spawn()
        .map_err(Error::io(format!("start {program}")))?;
    let limits = Limits {
        stdout: 32 << 20,
        stderr: 64 << 10,
    };
    let done = Box::pin(finish(child, None, limits)).await;
    let done = done.map_err(Error::io(format!("run {program}")))?;
    let fail = |detail: String| Error::Command {
        program,
        args: excerpt(args.as_bytes(), 200),
        detail,
    };
    if done.stdout_overflow {
        return Err(fail("output exceeds the size limit".into()));
    }
    if !done.status.success() {
        let stderr = excerpt(&done.stderr, 500);
        return Err(fail(format!("{}: {}", done.status, stderr.trim())));
    }
    Ok(done.stdout)
}
