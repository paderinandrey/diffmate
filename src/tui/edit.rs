#[rustfmt::skip]
mod body {
use crate::error::Error;
use crate::paths::create_private_dir_async;
use std::io::{self, stdout};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub enum Piece {
    Finding { id: String, title: String, body: String },
    Note { draft_id: i64, index: usize, text: String },
}
pub enum Edited {
    Same,
    Finding(String, String),
    Note(String),
    Message(&'static str),
}
const FAILED: &str = "the editor could not be started";
const SHAPE: &str = "the editor file must be a title, a blank line, and a body";
static EDITOR_FILE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub async fn run(editor: Option<&str>, cache: &Path, piece: &Piece, live: bool) -> Result<Edited, Error> {
    let Some(editor) = editor.filter(|value| value.bytes().any(|byte| !byte.is_ascii_whitespace())) else {
        return Ok(Edited::Message("EDITOR is not set"));
    };
    let argv: Vec<String> = editor.split(|c: char| c.is_ascii_whitespace()).filter(|part| !part.is_empty()).map(String::from).collect();
    let Some(path) = named(cache, piece) else { return Ok(Edited::Message(FAILED)); };
    let original = bytes(piece);
    if let Err(error) = write_private(&path, &original).await {
        return Err(Error::io("write the editor file")(error));
    }
    let held = live.then(Hold::suspend);
    let outcome = spawned(&argv, &path).await;
    drop(held);
    let saved = match outcome.as_ref() {
        Ok(status) if status.success() => tokio::fs::read(&path).await.ok(),
        _ => None,
    };
    let _ = tokio::fs::remove_file(&path).await;
    Ok(match outcome {
        Err(_) => Edited::Message(FAILED),
        Ok(status) if !status.success() => Edited::Same,
        Ok(_) => saved.map_or(Edited::Message(FAILED), |got| changed(piece, &original, &got)),
    })
}
fn changed(piece: &Piece, original: &[u8], got: &[u8]) -> Edited {
    if got == original || got.iter().all(u8::is_ascii_whitespace) { return Edited::Same; }
    let Ok(text) = std::str::from_utf8(got) else {
        return if matches!(piece, Piece::Note { .. }) { Edited::Same } else { Edited::Message(SHAPE) };
    };
    match (piece, text.split_once("\n\n")) {
        (Piece::Note { .. }, _) => Edited::Note(text.to_string()),
        (_, Some((title, body))) if !title.is_empty() => Edited::Finding(title.to_string(), body.to_string()),
        _ => Edited::Message(SHAPE),
    }
}
fn named(cache: &Path, piece: &Piece) -> Option<PathBuf> {
    let name = match piece {
        Piece::Finding { id, .. } => id.clone(),
        Piece::Note { draft_id, index, .. } => format!("note-{draft_id}-{index}"),
    };
    let bad = name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\']);
    let n = EDITOR_FILE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let id = std::process::id();
    (!bad).then(|| cache.join(format!("{name}-{id}-{n}")))
}
fn bytes(piece: &Piece) -> Vec<u8> {
    match piece {
        Piece::Finding { title, body, .. } => format!("{title}\n\n{body}").into_bytes(),
        Piece::Note { text, .. } => text.as_bytes().to_vec(),
    }
}
async fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() { create_private_dir_async(parent.to_path_buf()).await?; }
    let mut file = tokio::fs::OpenOptions::new().write(true).create_new(true).mode(0o700).open(path).await?;
    file.write_all(bytes).await?;
    file.flush().await
}
async fn spawned(argv: &[String], path: &Path) -> io::Result<std::process::ExitStatus> {
    let Some((program, rest)) = argv.split_first() else { return Err(io::Error::other("editor")); };
    Command::new(program).args(rest).arg(path).stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit()).spawn()?.wait().await
}
struct Hold;
impl Hold { fn suspend() -> Self { ratatui::restore(); Self } }
impl Drop for Hold {
    fn drop(&mut self) {
        let _ = ratatui::crossterm::terminal::enable_raw_mode();
        let _ = ratatui::crossterm::execute!(stdout(), ratatui::crossterm::terminal::EnterAlternateScreen);
    }
}
}
pub(super) use body::{Edited, Piece, run};
