use super::{ConfigError, Settings};
use serde::Deserialize;
use std::path::Path;
use toml::Spanned;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Raw {
    run_timeout_minutes: Option<Spanned<i64>>,
    debounce_seconds: Option<Spanned<i64>>,
    poll_interval_seconds: Option<Spanned<i64>>,
    max_retries: Option<Spanned<i64>>,
}

impl Raw {
    pub fn settings(self, text: &str, path: &Path) -> Result<Settings, ConfigError> {
        let mut settings = Settings::default();
        for (key, value, dest, maximum) in [
            (
                "run_timeout_minutes",
                self.run_timeout_minutes,
                &mut settings.run_timeout_minutes,
                u64::MAX / 60,
            ),
            (
                "debounce_seconds",
                self.debounce_seconds,
                &mut settings.debounce_seconds,
                u64::MAX,
            ),
            (
                "poll_interval_seconds",
                self.poll_interval_seconds,
                &mut settings.poll_interval_seconds,
                u64::MAX,
            ),
        ] {
            if let Some(value) = value {
                *dest = u64::try_from(*value.get_ref())
                    .ok()
                    .filter(|v| (1..=maximum).contains(v))
                    .ok_or_else(|| {
                        ConfigError::value(
                            path,
                            text,
                            key,
                            value.span().start,
                            &format!("must be a whole number between 1 and {maximum}"),
                        )
                    })?;
            }
        }
        if let Some(value) = self.max_retries {
            settings.max_retries = u32::try_from(*value.get_ref()).map_err(|_| {
                ConfigError::value(
                    path,
                    text,
                    "max_retries",
                    value.span().start,
                    "must be a whole number between 0 and 4294967295",
                )
            })?;
        }
        Ok(settings)
    }
}
