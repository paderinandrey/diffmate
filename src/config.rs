mod error;
mod raw;
#[cfg(test)]
mod tests;

pub(crate) use error::ConfigError;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Settings {
    run_timeout_minutes: u64,
    debounce_seconds: u64,
    poll_interval_seconds: u64,
    max_retries: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            run_timeout_minutes: 15,
            debounce_seconds: 60,
            poll_interval_seconds: 60,
            max_retries: 2,
        }
    }
}

impl Settings {
    pub fn run_timeout(&self) -> Duration {
        Duration::from_secs(self.run_timeout_minutes * 60)
    }

    pub fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

pub(crate) fn load(path: &Path) -> Result<Settings, ConfigError> {
    match std::fs::read_to_string(path) {
        Ok(text) => parse(&text, path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(error) => Err(ConfigError::io(path, error)),
    }
}

pub(crate) fn parse(text: &str, path: &Path) -> Result<Settings, ConfigError> {
    let raw: raw::Raw =
        toml::from_str(text).map_err(|error| ConfigError::parse(path, text, error))?;
    raw.settings(text, path)
}
