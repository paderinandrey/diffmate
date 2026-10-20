use crate::pr_url::UrlError;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("HOME is not set")]
    NoHome,
    #[error("git was not found in PATH")]
    NoGit,
    #[error("{what}: {source}")]
    Io { what: String, source: io::Error },
    #[error("agent input {0} does not exist")]
    Missing(PathBuf),
    #[error("{program} {args}: {detail}")]
    Command {
        program: &'static str,
        args: String,
        detail: String,
    },
    #[error("database: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("database schema version {0} is newer than this diffmate supports")]
    SchemaVersion(i64),
    #[error("background task failed: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("unexpected JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("output schema: {0}")]
    Schema(String),
    #[error("{0}")]
    Url(#[from] UrlError),
    #[error("PR resolves to {0}; run diffmate review with that URL")]
    Canonical(String),
}

impl Error {
    pub(crate) fn io(what: impl Into<String>) -> impl FnOnce(io::Error) -> Error {
        let what = what.into();
        move |source| Error::Io { what, source }
    }
}
