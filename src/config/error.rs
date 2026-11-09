mod key;

use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct ConfigError {
    pub path: PathBuf,
    pub line: Option<usize>,
    pub key: Option<String>,
    pub message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl ConfigError {
    pub(super) fn io(path: &Path, error: std::io::Error) -> Self {
        Self {
            path: path.to_path_buf(),
            line: None,
            key: None,
            message: error.to_string(),
            source: Some(Box::new(error)),
        }
    }

    pub(super) fn value(path: &Path, text: &str, key: &str, start: usize, message: &str) -> Self {
        Self {
            path: path.to_path_buf(),
            line: Some(line(text, start)),
            key: Some(key.to_owned()),
            message: message.to_owned(),
            source: None,
        }
    }

    pub(super) fn parse(path: &Path, text: &str, error: toml::de::Error) -> Self {
        let line = error.span().map(|span| line(text, span.start));
        let message = error.message().to_owned();
        let key = key::name(&error, text, line);
        Self {
            path: path.to_path_buf(),
            line,
            key,
            message,
            source: Some(Box::new(error)),
        }
    }
}

fn line(text: &str, start: usize) -> usize {
    text.as_bytes()[..start.min(text.len())]
        .split(|byte| *byte == b'\n')
        .count()
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())?;
        if let Some(line) = self.line {
            write!(f, ":{line}")?;
        }
        write!(f, ": ")?;
        if let Some(key) = &self.key {
            write!(f, "{key}: ")?;
        }
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|source| &**source as _)
    }
}
