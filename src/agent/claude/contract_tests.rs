use super::tests::{contract, inherited, request};
use super::{AUTH_ENV, ClaudeCli, user_prompt};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

fn strings(value: &Value) -> Vec<String> {
    let items = value.as_array().unwrap();
    items
        .iter()
        .map(|item| item.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn allowlist_matches_spike() {
    let recorded: BTreeSet<String> = strings(&contract()["auth_env_allowlist"])
        .into_iter()
        .collect();
    let ours: BTreeSet<String> = AUTH_ENV.iter().map(|name| (*name).to_string()).collect();
    assert_eq!(ours, recorded);
}

#[test]
fn production_vector_matches_spike() {
    let cli = ClaudeCli::new(&inherited(&[])).unwrap();
    let request = request();
    let expected: Vec<String> = strings(&contract()["production_flags"])
        .into_iter()
        .map(|flag| match flag.as_str() {
            "CONTEXT_DIR" => request.context_dir.to_str().unwrap().to_string(),
            "PROMPT_FILE" => request.prompt_file.to_str().unwrap().to_string(),
            "CLI_SCHEMA" => cli.schemas.cli_arg.clone(),
            _ => flag,
        })
        .collect();
    let actual: Vec<String> = cli
        .args_for(&request)
        .iter()
        .map(|arg| arg.to_str().unwrap().to_string())
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn user_prompt_names_only_the_context_dir() {
    let (first, second) = (Path::new("/runs/1/context"), Path::new("/runs/22/context"));
    let (a, b) = (user_prompt(first), user_prompt(second));
    assert!(a.contains("/runs/1/context") && b.contains("/runs/22/context"));
    assert_eq!(
        a.replace("/runs/1/context", "@"),
        b.replace("/runs/22/context", "@")
    );
}
