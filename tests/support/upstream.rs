use super::git_in;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

pub fn real_git() -> PathBuf {
    let path = std::env::var_os("PATH").unwrap();
    let mut candidates = std::env::split_paths(&path).map(|dir| dir.join("git"));
    candidates.find(|candidate| candidate.is_file()).unwrap()
}

pub struct Upstream {
    pub bare: PathBuf,
    pub work: PathBuf,
    home: PathBuf,
}

impl Upstream {
    pub fn create(upstreams: &Path, owner: &str, repo: &str) -> Upstream {
        let bare = upstreams.join(owner).join(format!("{repo}.git"));
        let work = upstreams.join(".work").join(owner).join(repo);
        let home = upstreams.join(".home");
        for dir in [&bare, &work, &home] {
            fs::create_dir_all(dir).unwrap();
        }
        git_in(&bare, &home, &["init", "-q", "--bare", "-b", "main"]);
        git_in(&work, &home, &["init", "-q", "-b", "main"]);
        Upstream { bare, work, home }
    }

    pub fn git(&self, args: &[&str]) -> String {
        git_in(&self.work, &self.home, args)
    }

    fn commit_all(&self, message: &str) -> String {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", message]);
        self.git(&["rev-parse", "HEAD"])
    }

    pub fn commit(&self, files: &[(&str, &str)], message: &str) -> String {
        for (path, content) in files {
            let target = self.work.join(path);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::write(target, content).unwrap();
        }
        self.commit_all(message)
    }

    pub fn commit_symlink(&self, path: &str, target: &str, message: &str) -> String {
        symlink(target, self.work.join(path)).unwrap();
        self.commit_all(message)
    }

    pub fn remove(&self, paths: &[&str], message: &str) -> String {
        for path in paths {
            fs::remove_file(self.work.join(path)).unwrap();
        }
        self.commit_all(message)
    }

    fn push(&self, refname: &str) {
        let target = format!("HEAD:{refname}");
        self.git(&["push", "-q", self.bare.to_str().unwrap(), &target]);
    }

    pub fn push_branch(&self, branch: &str) {
        self.push(&format!("refs/heads/{branch}"));
    }

    pub fn pr_head(&self, number: u64, base_branch: &str, files: &[(&str, &str)]) -> String {
        let base = format!("refs/heads/{base_branch}");
        self.git(&["checkout", "-q", "--detach", &base]);
        let sha = self.commit(files, &format!("pr {number}"));
        self.push(&format!("refs/pull/{number}/head"));
        self.git(&["checkout", "-q", base_branch]);
        sha
    }
}
