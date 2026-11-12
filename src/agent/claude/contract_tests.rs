use super::tests::{contract, inherited, request, rules};
use super::{AUTH_ENV, ClaudeCli, user_prompt};
use crate::agent::probe::HELP_FLAGS;
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
    let rules = rules(&request);
    let expected: Vec<String> = strings(&contract()["production_flags"])
        .into_iter()
        .map(|flag| match flag.as_str() {
            "PROMPT" => user_prompt(&request.context_dir),
            "RULES" => rules.clone(),
            "CLI_SCHEMA" => cli.schemas.cli_arg.clone(),
            _ => flag,
        })
        .collect();
    let actual: Vec<String> = cli
        .args_for(&request, &rules)
        .iter()
        .map(|arg| arg.to_str().unwrap().to_string())
        .collect();
    assert_eq!(actual, expected);
    for flag in HELP_FLAGS {
        let present = if *flag == "--single" {
            actual.iter().any(|arg| arg == "-p")
        } else {
            actual.iter().any(|arg| arg == *flag)
        };
        assert!(present, "{flag}");
    }
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
