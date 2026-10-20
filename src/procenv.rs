use std::ffi::{OsStr, OsString};

pub(crate) type Pairs = Vec<(OsString, OsString)>;

const TOOL_VARS: [&str; 10] = [
    "USER",
    "TMPDIR",
    "XDG_CONFIG_HOME",
    "GH_CONFIG_DIR",
    "GH_TOKEN",
    "GITHUB_TOKEN",
    "HTTPS_PROXY",
    "HTTP_PROXY",
    "NO_PROXY",
    "SSL_CERT_FILE",
];

#[derive(Debug, Clone)]
pub(crate) struct InheritedEnv(Pairs);

impl InheritedEnv {
    pub fn capture() -> InheritedEnv {
        InheritedEnv::from_pairs(std::env::vars_os().collect())
    }

    pub fn from_pairs(pairs: Pairs) -> InheritedEnv {
        InheritedEnv(pairs)
    }

    pub fn get(&self, name: &str) -> Option<&OsStr> {
        let found = self.0.iter().find(|(key, _)| key == name);
        found.map(|(_, value)| value.as_os_str())
    }

    pub fn pick(&self, names: &[&str]) -> Pairs {
        let pick_one = |name: &&str| {
            let value = self.get(name)?;
            Some((OsString::from(name), value.to_os_string()))
        };
        names.iter().filter_map(pick_one).collect()
    }

    pub fn tool_env(&self) -> Pairs {
        let mut pairs = self.pick(&["PATH", "HOME"]);
        pairs.push(("LC_ALL".into(), "C".into()));
        pairs.extend(self.pick(&TOOL_VARS));
        pairs
    }
}
