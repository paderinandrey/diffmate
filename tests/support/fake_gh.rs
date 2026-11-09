use super::fake_call::Hold;
use super::{call_numbers, mkfifo, write_executable};
use std::fs;
use std::path::{Path, PathBuf};

pub struct FakeGh {
    pub dir: PathBuf,
}

impl FakeGh {
    pub fn install(bin: &Path, dir: &Path) -> FakeGh {
        fs::create_dir_all(dir.join("calls")).unwrap();
        fs::create_dir_all(dir.join("routes")).unwrap();
        let script = include_str!("fake_gh.sh").replace("@STATE@", dir.to_str().unwrap());
        write_executable(&bin.join("gh"), &script);
        FakeGh {
            dir: dir.to_path_buf(),
        }
    }

    fn add(&self, prefix: &str, stdout: &str) -> PathBuf {
        let routes = self.dir.join("routes");
        let index = fs::read_dir(&routes).unwrap().count();
        let stem = routes.join(format!("{index:04}"));
        fs::write(stem.with_extension("route"), format!("{prefix}\n{stdout}")).unwrap();
        stem
    }

    pub fn route(&self, prefix: &str, stdout: &str) {
        self.add(prefix, stdout);
    }

    pub fn route_hold(&self, prefix: &str, stdout: &str) -> Hold {
        let stem = self.add(prefix, stdout);
        let started = self.dir.join("started");
        let release = self.dir.join("release");
        mkfifo(&started);
        mkfifo(&release);
        fs::write(stem.with_extension("hold"), "").unwrap();
        Hold { started, release }
    }

    pub fn route_fail(&self, prefix: &str, stderr: &str, exit: i32) {
        let stem = self.add(prefix, "");
        fs::write(stem.with_extension("stderr"), stderr).unwrap();
        fs::write(stem.with_extension("exit"), exit.to_string()).unwrap();
    }

    pub fn replace_route(&self, prefix: &str, stdout: &str) {
        let stem = self.find_route(prefix);
        fs::write(&stem, format!("{prefix}\n{stdout}")).unwrap();
        for extension in ["stderr", "exit"] {
            let path = stem.with_extension(extension);
            if path.exists() {
                fs::remove_file(path).unwrap();
            }
        }
    }

    pub fn replace_route_fail(&self, prefix: &str, stderr: &str, exit: i32) {
        self.replace_route(prefix, "");
        let stem = self.find_route(prefix);
        fs::write(stem.with_extension("stderr"), stderr).unwrap();
        fs::write(stem.with_extension("exit"), exit.to_string()).unwrap();
    }

    fn find_route(&self, prefix: &str) -> PathBuf {
        fs::read_dir(self.dir.join("routes"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| {
                path.extension().is_some_and(|ext| ext == "route")
                    && fs::read_to_string(path).unwrap().lines().next() == Some(prefix)
            })
            .unwrap_or_else(|| panic!("no route for {prefix}"))
    }

    pub fn calls(&self) -> Vec<Vec<String>> {
        let calls = self.dir.join("calls");
        call_numbers(&calls)
            .into_iter()
            .map(|n| {
                let argv =
                    fs::read_to_string(calls.join(n.to_string()).join("argv")).unwrap_or_default();
                argv.lines().map(String::from).collect()
            })
            .collect()
    }
}

pub fn pr_view_json(
    owner: &str,
    repo: &str,
    number: u64,
    head_sha: &str,
    base_ref: &str,
    cross_repository: bool,
) -> String {
    format!(
        "{{\"number\":{number},\"url\":\"https://github.com/{owner}/{repo}/pull/{number}\",\"title\":\"Fixture PR\",\"body\":\"Fixture body\",\"author\":{{\"login\":\"fixture-author\"}},\"baseRefName\":\"{base_ref}\",\"headRefName\":\"feature\",\"headRefOid\":\"{head_sha}\",\"isCrossRepository\":{cross_repository},\"state\":\"OPEN\"}}"
    )
}
