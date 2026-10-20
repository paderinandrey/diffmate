use crate::error::Error;
use crate::gh::PrMeta;
use crate::paths::create_private_dir_async;
use crate::pr_url::PrRef;
use crate::store::records::Revision;
use std::path::Path;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

const PROMPT: &str = include_str!("../prompts/review.md");

async fn put(path: &Path, bytes: impl AsRef<[u8]>) -> Result<(), Error> {
    let what = format!("write {}", path.display());
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(0o600);
    let mut file = options.open(path).await.map_err(Error::io(&what))?;
    file.write_all(bytes.as_ref())
        .await
        .map_err(Error::io(&what))?;
    file.flush().await.map_err(Error::io(&what))
}

fn pr_md(pr: &PrRef, meta: &PrMeta, revision: &Revision) -> String {
    let fork = if meta.is_cross_repository {
        "yes"
    } else {
        "no"
    };
    format!(
        "# {label} (#{number})\n\nTitle: {title}\nAuthor: {author}\nState: {state}\nURL: {url}\nBase branch: {base}\nHead branch: {head}\nHead SHA: {head_sha}\nMerge-base SHA: {merge_base}\nHead is in another repository: {fork}\n\n## Description\n\n{body}\n",
        label = pr.label(),
        number = meta.number,
        title = meta.title,
        author = meta.author,
        state = meta.state,
        url = meta.url,
        base = meta.base_ref,
        head = meta.head_ref,
        head_sha = revision.head_sha,
        merge_base = revision.merge_base_sha,
        body = meta.body,
    )
}

pub(crate) async fn write(
    dir: &Path,
    pr: &PrRef,
    meta: &PrMeta,
    revision: &Revision,
    diff: Vec<u8>,
) -> Result<(), Error> {
    let made = create_private_dir_async(dir.to_path_buf()).await;
    made.map_err(Error::io("create context directory"))?;
    put(&dir.join("pr.md"), pr_md(pr, meta, revision)).await?;
    put(&dir.join("diff.patch"), diff).await
}

pub(crate) async fn write_prompt(path: &Path) -> Result<(), Error> {
    let dir = path.parent().unwrap_or(path).to_path_buf();
    let made = create_private_dir_async(dir).await;
    made.map_err(Error::io("create prompt directory"))?;
    put(path, PROMPT).await
}
