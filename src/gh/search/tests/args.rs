use super::super::search_args;

#[test]
fn search_args_request_the_owner_and_not_a_team() {
    let args = search_args(100);
    assert!(args.iter().any(|arg| arg == "--review-requested=@me"));
    assert!(args.iter().any(|arg| arg == "--state=open"));
    let fields = args.iter().find(|arg| arg.contains("isDraft")).unwrap();
    for field in ["number", "repository", "url", "isDraft", "state"] {
        assert!(fields.split(',').any(|part| part == field));
    }
    assert!(!args.iter().any(|arg| arg == "--draft"));
    assert!(!args.iter().any(|arg| arg.starts_with("--team")));
    assert_eq!(args.first().map(String::as_str), Some("search"));
    assert_eq!(args.get(1).map(String::as_str), Some("prs"));
    let limit = args.iter().position(|arg| arg == "--limit").unwrap();
    assert_eq!(args.get(limit + 1).map(String::as_str), Some("100"));
}
