use super::ClaudeCli;
use super::tests::inherited;
use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn agent_env_is_the_allowlist() {
    let secrets = [
        ("GH_TOKEN", "t"),
        ("GITHUB_TOKEN", "t"),
        ("CLAUDECODE", "1"),
        ("GIT_DIR", "/elsewhere"),
        ("AWS_SECRET_ACCESS_KEY", "k"),
        ("ANTHROPIC_BASE_URL", "http://evil"),
    ];
    let allowed = [
        ("HOME", "/h"),
        ("USER", "u"),
        ("LANG", "C"),
        ("ANTHROPIC_API_KEY", "a"),
    ];
    let all: Vec<_> = secrets.iter().chain(&allowed).copied().collect();
    let cli = ClaudeCli::new(&inherited(&all)).unwrap();

    let names = |pairs: &[(std::ffi::OsString, std::ffi::OsString)]| {
        pairs
            .iter()
            .map(|(name, _)| name.to_str().unwrap().to_string())
            .collect::<BTreeSet<_>>()
    };
    let mut expected: BTreeSet<String> = allowed
        .iter()
        .map(|(name, _)| (*name).to_string())
        .collect();
    expected.extend(["PATH", "GIT_CONFIG_GLOBAL", "GIT_CONFIG_NOSYSTEM"].map(String::from));
    assert_eq!(names(cli.env_vars()), expected);
    let value_of = |name: &str| {
        cli.env_vars()
            .iter()
            .find(|(key, _)| key == name)
            .unwrap()
            .1
            .clone()
    };
    assert_eq!(value_of("GIT_CONFIG_GLOBAL"), "/dev/null");
    assert_eq!(value_of("GIT_CONFIG_NOSYSTEM"), "1");
    let run = cli.run_env(Path::new("/run/prompt/bin")).unwrap();
    assert_eq!(names(&run), expected);
    let path = run.iter().find(|(key, _)| key == "PATH").unwrap().1.clone();
    assert_eq!(
        std::env::split_paths(&path).next().unwrap(),
        Path::new("/run/prompt/bin")
    );
}
