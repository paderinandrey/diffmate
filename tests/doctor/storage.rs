use super::support::sandbox::Sandbox;
use std::fs;
use std::os::unix::fs::PermissionsExt;

#[test]
fn loose_state_dir_and_file_cache_share_one_storage_line() {
    let sandbox = Sandbox::new();
    sandbox.gh.route("auth status", "logged in\n");
    let state = sandbox.state_dir();
    let cache = sandbox.cache_dir();
    fs::create_dir_all(&state).unwrap();
    fs::set_permissions(&state, fs::Permissions::from_mode(0o755)).unwrap();
    fs::create_dir_all(cache.parent().unwrap()).unwrap();
    fs::write(&cache, b"not a directory\n").unwrap();

    let output = sandbox.run(&["doctor"]);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.len(), 5, "{stdout}");
    let storage = lines[2];
    assert!(storage.starts_with("storage:"), "{storage}");
    assert!(storage.contains(&state.display().to_string()), "{storage}");
    assert!(storage.contains("chmod 700"), "{storage}");
    assert!(storage.contains(&cache.display().to_string()), "{storage}");
    assert!(storage.contains("is not a directory"), "{storage}");
    assert_eq!(
        fs::metadata(&state).unwrap().permissions().mode() & 0o777,
        0o755
    );
    assert!(cache.is_file());
    assert_eq!(fs::read(&cache).unwrap(), b"not a directory\n");
}
