use crate::error::Error;
use crate::paths::create_private_dir_async;
use std::path::Path;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

const PROMPT: &str = include_str!("../../prompts/review.md");

pub(super) async fn put(path: &Path, bytes: impl AsRef<[u8]>) -> Result<(), Error> {
    let what = format!("write {}", path.display());
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(0o600);
    let mut file = options.open(path).await.map_err(Error::io(&what))?;
    file.write_all(bytes.as_ref())
        .await
        .map_err(Error::io(&what))?;
    file.flush().await.map_err(Error::io(&what))
}

pub(crate) async fn write_prompt(path: &Path) -> Result<(), Error> {
    let dir = path.parent().unwrap_or(path).to_path_buf();
    let made = create_private_dir_async(dir).await;
    made.map_err(Error::io("create prompt directory"))?;
    put(path, PROMPT).await
}
