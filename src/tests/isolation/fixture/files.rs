use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub(super) fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

pub(super) fn script(path: &Path, body: &str) {
    write(path, &format!("#!/bin/sh\n{body}\n"));
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}
