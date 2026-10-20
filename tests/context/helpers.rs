use crate::support::fake_claude::{finding_json, output_json, success_envelope};
use crate::support::sandbox::Sandbox;
use std::fs;
use std::path::PathBuf;

pub const GRAPHQL: &str = "api graphql";
pub const CHECK_RUNS: &str = "api repos/o/r/commits/";
pub const ANNOTATIONS: &str = "api repos/o/r/check-runs/77/annotations";

const PAGE: &str = "\"pageInfo\":{\"hasNextPage\":false}";
const THREAD_COMMENT: &str = "{\"author\":{\"login\":\"bob\"},\"body\":\"Fixed in the next commit.\",\"createdAt\":\"2026-01-01T00:00:00Z\"}";

pub fn discussion_json() -> String {
    let comment = "{\"author\":{\"login\":\"alice\"},\"body\":\"Please check the bound.\",\"createdAt\":\"2026-01-01T00:00:00Z\"}";
    let thread = format!(
        "{{\"isResolved\":true,\"isOutdated\":false,\"path\":\"src/lib.rs\",\"line\":3,\"comments\":{{{PAGE},\"nodes\":[{THREAD_COMMENT}]}}}}"
    );
    let pull = format!(
        "\"comments\":{{{PAGE},\"nodes\":[{comment}]}},\"reviews\":{{{PAGE},\"nodes\":[]}},\"reviewThreads\":{{{PAGE},\"nodes\":[{thread}]}}"
    );
    format!("{{\"data\":{{\"repository\":{{\"pullRequest\":{{{pull}}}}}}}}}")
}

pub fn check_runs_json() -> String {
    let failed = "{\"id\":77,\"name\":\"build\",\"status\":\"completed\",\"conclusion\":\"failure\",\"output\":{\"title\":\"Build failed\",\"summary\":null}}";
    let passed = "{\"id\":78,\"name\":\"lint\",\"status\":\"completed\",\"conclusion\":\"success\",\"output\":{\"title\":null,\"summary\":null}}";
    format!("{{\"total_count\":2,\"check_runs\":[{failed},{passed}]}}")
}

pub const ANNOTATIONS_JSON: &str = "[{\"path\":\"src/lib.rs\",\"start_line\":3,\"end_line\":3,\"annotation_level\":\"failure\",\"title\":\"t\",\"message\":\"mismatched types\"}]";

pub fn add_github_routes(sandbox: &Sandbox) {
    sandbox.gh.route(GRAPHQL, &discussion_json());
    sandbox.gh.route(ANNOTATIONS, ANNOTATIONS_JSON);
    sandbox.gh.route(CHECK_RUNS, &check_runs_json());
}

pub fn captured(sandbox: &Sandbox, nth_run: usize) -> PathBuf {
    let calls = sandbox.claude.run_calls();
    calls[nth_run].context.clone().unwrap()
}

pub fn read(context: &std::path::Path, name: &str) -> String {
    fs::read_to_string(context.join(name)).unwrap()
}

pub fn finding(previous: &str) -> String {
    let (path, side) = (Some("src/lib.rs"), Some("RIGHT"));
    let json = finding_json(path, Some(3), side, "high", "Off by one", "Body.");
    json.replace("\"previous_finding_id\":null", previous)
}

pub fn set_output(sandbox: &Sandbox, finding: String) {
    let output = output_json(&[finding], None, &[]);
    sandbox.claude.set_run(&success_envelope(&output), 0);
}

pub fn assert_read_only(calls: &[Vec<String>]) {
    let kinds: Vec<_> = calls.iter().map(|argv| argv[..2].join(" ")).collect();
    assert_eq!(
        kinds
            .iter()
            .filter(|kind| kind.as_str() == "api graphql")
            .count(),
        1
    );
    assert!(
        kinds
            .iter()
            .any(|kind| kind.contains("check-runs?per_page"))
    );
    assert!(
        kinds
            .iter()
            .any(|kind| kind.ends_with("/annotations?per_page=50"))
    );
    for argv in calls {
        match (argv[0].as_str(), argv[1].as_str()) {
            ("pr", "view") => {}
            ("api", "graphql") => {
                let query = argv.iter().find(|arg| arg.starts_with("query=")).unwrap();
                assert!(!query.to_lowercase().contains("mutation"), "{query}");
            }
            ("api", endpoint) => {
                assert!(endpoint.starts_with("repos/o/r/"), "{argv:?}");
                let writes = ["-X", "--method", "-f", "-F"];
                assert!(
                    argv.iter().all(|arg| !writes.contains(&arg.as_str())),
                    "{argv:?}"
                );
            }
            other => panic!("unexpected gh call {other:?}"),
        }
    }
}
