use super::Check;
use super::which::which;
use crate::agent::claude::AUTH_ENV;
use crate::agent::probe::{check_flags, check_version};
use crate::app::AppConfig;
use std::ffi::OsStr;

pub(super) async fn check(config: &AppConfig) -> Check {
    let path = config.env.get("PATH").unwrap_or_default();
    if which(path, "grok").await.is_none() {
        let path = path.to_string_lossy();
        return Check::fail(format!("grok: grok was not found in PATH: {path}"));
    }
    let env = config.env.pick(&AUTH_ENV);
    let program = OsStr::new("grok");
    match check_version(program, &env, None).await {
        Ok(Some(())) => {}
        Ok(None) => return Check::fail("grok: probe did not finish"),
        Err(detail) => return Check::fail(format!("grok: {}", detail.message)),
    }
    match check_flags(program, &env, None).await {
        Ok(Some(())) => Check::pass("grok: ok"),
        Ok(None) => Check::fail("grok: probe did not finish"),
        Err(detail) => Check::fail(format!("grok: {}", detail.message)),
    }
}
