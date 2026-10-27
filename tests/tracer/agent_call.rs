use crate::support::sandbox::Sandbox;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;

pub fn assert_agent_call(sandbox: &Sandbox) {
    let calls = sandbox.claude.run_calls();
    assert_eq!(calls.len(), 1);
    let call = &calls[0];
    let run_dir = sandbox.cache_dir().join("runs/1");
    let context = run_dir.join("context");
    let contract: Value =
        serde_json::from_str(include_str!("../fixtures/claude/contract.json")).unwrap();
    let mut schema: Value =
        serde_json::from_str(include_str!("../../schemas/review.schema.json")).unwrap();
    schema.as_object_mut().unwrap().remove("$schema");
    let substitute = |flag: &str| match flag {
        "CONTEXT_DIR" => context.to_str().unwrap().to_string(),
        "PROMPT_FILE" => run_dir
            .join("prompt/review.md")
            .to_str()
            .unwrap()
            .to_string(),
        "CLI_SCHEMA" => schema.to_string(),
        other => other.to_string(),
    };
    let expected: Vec<String> = contract["production_flags"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| substitute(f.as_str().unwrap()))
        .collect();
    assert_eq!(call.argv, expected);
    assert_eq!(call.cwd, run_dir.join("worktree"));
    let copy = call.context.as_ref().unwrap();
    assert!(
        fs::read_to_string(copy.join("diff.patch"))
            .unwrap()
            .contains("+fn c() { 1 }")
    );
    assert!(copy.join("pr.md").exists());
    let stdin = String::from_utf8(call.stdin.clone()).unwrap();
    assert!(stdin.contains(context.to_str().unwrap()));
    assert!(!stdin.contains("Fixture"));
    let env: BTreeMap<_, _> = call.env.iter().cloned().collect();
    assert_eq!(env["GIT_CONFIG_GLOBAL"], "/dev/null");
    assert_eq!(env["GIT_CONFIG_NOSYSTEM"], "1");
    assert!(env["PATH"].starts_with(run_dir.join("prompt/bin").to_str().unwrap()));
    let mut allowed: Vec<&str> = contract["auth_env_allowlist"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap())
        .collect();
    allowed.extend([
        "GIT_CONFIG_GLOBAL",
        "GIT_CONFIG_NOSYSTEM",
        "PWD",
        "SHLVL",
        "OLDPWD",
        "_",
    ]);
    assert!(
        env.keys().all(|key| allowed.contains(&key.as_str())),
        "{env:?}"
    );
}
