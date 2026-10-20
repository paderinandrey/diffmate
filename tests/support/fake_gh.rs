use super::{call_numbers, write_executable};
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

    pub fn route_fail(&self, prefix: &str, stderr: &str, exit: i32) {
        let stem = self.add(prefix, "");
        fs::write(stem.with_extension("stderr"), stderr).unwrap();
        fs::write(stem.with_extension("exit"), exit.to_string()).unwrap();
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
