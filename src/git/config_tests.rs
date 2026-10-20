use super::Git;
use super::fixture::git;
use super::tests::clone;
use std::fs;

const KEYS: [&str; 9] = [
    "core.bare",
    "core.filemode",
    "core.fsmonitor",
    "core.hookspath",
    "core.repositoryformatversion",
    "core.symlinks",
    "gc.autodetach",
    "maintenance.autodetach",
    "remote.origin.url",
];
const PLATFORM_KEYS: [&str; 3] = [
    "core.ignorecase",
    "core.logallrefupdates",
    "core.precomposeunicode",
];

#[tokio::test]
async fn local_config_has_exactly_the_written_keys_on_every_run() {
    let clone = clone();
    let again = || {
        clone
            .cli
            .ensure_clone(&clone.repo, "file:///x", &clone.hooks)
    };
    again().await.unwrap();
    git(
        &clone.repo,
        &["config", "--add", "remote.origin.fetch", "+a:b"],
    );
    again().await.unwrap();
    let listing = git(&clone.repo, &["config", "--local", "--name-only", "--list"]);
    let mut keys: Vec<_> = listing.lines().map(str::to_lowercase).collect();
    keys.retain(|key| !PLATFORM_KEYS.contains(&key.as_str()));
    keys.sort();
    assert_eq!(keys, KEYS);
    assert!(fs::read_dir(&clone.hooks).unwrap().next().is_none());
}
