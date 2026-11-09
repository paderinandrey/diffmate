use crate::config::Settings;
use crate::error::Error;
use crate::paths::Paths;
use crate::procenv::InheritedEnv;
use crate::store::now_ms;
use std::path::{Path, PathBuf};

pub(crate) struct ProcessInfo {
    pub pid: u32,
    pub started_at_ms: i64,
}

pub(crate) struct AppConfig {
    pub paths: Paths,
    pub env: InheritedEnv,
    pub process: ProcessInfo,
    pub config_file: PathBuf,
    pub settings: Settings,
}

impl AppConfig {
    pub fn from_process() -> Result<AppConfig, Error> {
        let env = InheritedEnv::capture();
        let home = env.get("HOME").filter(|home| !home.is_empty());
        let home = Path::new(home.ok_or(Error::NoHome)?);
        let paths = Paths::from_home(home);
        let config_file = home.join(".config/diffmate/config.toml");
        let process = ProcessInfo {
            pid: std::process::id(),
            started_at_ms: now_ms(),
        };
        Ok(AppConfig {
            paths,
            env,
            process,
            config_file,
            settings: Settings::default(),
        })
    }
}
