use super::contract::contract;
use super::endpoint::Recorded;
use super::harness::EXPECTED_TOOLS;
use super::inspect::{assert_reached, bodies_contain, tool_names};

fn request(method: &str, path: &str, body: &str) -> Recorded {
    Recorded {
        method: method.into(),
        path: path.into(),
        body: body.as_bytes().to_vec(),
    }
}

#[test]
fn needle_search_runs_over_every_raw_body() {
    let seen = [
        request("HEAD", "/api/hello", ""),
        request("POST", "/v1/messages", "{\"a\":\"xx dmcanary0123 yy\"}"),
    ];
    assert!(bodies_contain(&seen, "dmcanary0123"));
    assert!(bodies_contain(&seen, "yy\"}"));
    assert!(!bodies_contain(&seen, "dmcanary9999"));
    assert!(!bodies_contain(&[], "x"));
}

#[test]
fn tool_names_are_collected_from_every_json_body() {
    let first = request(
        "POST",
        "/v1/messages",
        "{\"tools\":[{\"name\":\"Read\"},{\"name\":\"Grep\"}]}",
    );
    let second = request(
        "POST",
        "/v1/messages",
        "{\"tools\":[{\"name\":\"Read\"}],\"x\":1}",
    );
    let other = request("POST", "/x", "not json");
    let none = request("HEAD", "/api/hello", "");
    let names: Vec<_> = tool_names(&[first, second, other, none])
        .into_iter()
        .collect();
    assert_eq!(names, ["Grep", "Read"]);
}

#[test]
#[should_panic(expected = "never sent a model request")]
fn a_run_without_a_model_request_is_not_reached() {
    assert_reached(&[request("HEAD", "/api/hello", "")], "run");
}

#[test]
fn expected_tool_set_is_the_owner_accepted_set_from_the_contract() {
    let contract = contract();
    let recorded: Vec<_> = contract["isolated_first_look"]["tool_names"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| name.as_str().unwrap().to_string())
        .collect();
    let mut expected: Vec<_> = EXPECTED_TOOLS.iter().map(ToString::to_string).collect();
    expected.sort();
    let mut recorded = recorded;
    recorded.sort();
    assert_eq!(recorded, expected);
    assert!(
        expected.contains(
            &contract["structured_output_tool"]
                .as_str()
                .unwrap()
                .to_string()
        )
    );
}
