use super::Layout;
use super::files::script;
use std::path::PathBuf;

fn touch(layout: &Layout, marker: &str) -> String {
    format!(": > '{}'", layout.marker(marker).display())
}

fn real_git() -> PathBuf {
    let path = std::env::var_os("PATH").unwrap();
    let mut found = std::env::split_paths(&path).map(|dir| dir.join("git"));
    found.find(|candidate| candidate.is_file()).unwrap()
}

fn shim(layout: &Layout) -> String {
    format!(
        "printf 'GIT_CONFIG_GLOBAL=%s GIT_CONFIG_NOSYSTEM=%s argv: %s\\n' \"${{GIT_CONFIG_GLOBAL-<unset>}}\" \"${{GIT_CONFIG_NOSYSTEM-<unset>}}\" \"$*\" >> '{}'\nexec '{}' \"$@\"",
        layout.git_log.display(),
        real_git().display()
    )
}

pub(super) fn write(layout: &Layout) {
    let dir = &layout.scripts;
    let hook = touch(layout, "session_start_hook");
    script(&dir.join("hook.sh"), &hook);
    let helper = format!("{}\necho dummy-helper-key", touch(layout, "api_key_helper"));
    script(&dir.join("helper.sh"), &helper);
    script(&dir.join("plugin-hook.sh"), &touch(layout, "plugin_hook"));
    let mcp = format!("{}\nexit 1", touch(layout, "mcp_server"));
    script(&dir.join("mcp.sh"), &mcp);
    let filter = format!("{}\ncat", touch(layout, "git_filter"));
    script(&dir.join("filter.sh"), &filter);
    script(&layout.bin.join("git"), &shim(layout));
}
