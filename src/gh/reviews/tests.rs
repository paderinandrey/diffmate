use super::parse::{created_from, login_from, reviews_from};
use super::post_parts;
use crate::gh::{InlineComment, ReviewRequest};
use crate::pr_url::PrRef;

#[test]
fn create_review_keeps_the_body_off_the_path() {
    let pr = PrRef {
        owner: "o".into(),
        repo: "r".into(),
        number: 7,
    };
    let review = ReviewRequest {
        commit_id: "abc".into(),
        body: "secret body".into(),
        event: "COMMENT".into(),
        comments: vec![InlineComment {
            path: "a.rs".into(),
            line: 3,
            side: "RIGHT".into(),
            body: "inline".into(),
        }],
    };
    let (path, bytes) = post_parts(&pr, &review).unwrap();
    assert_eq!(path, "repos/o/r/pulls/7/reviews");
    assert!(!path.contains("secret") && !path.contains("inline"));
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["body"], "secret body");
    assert_eq!(value["comments"][0]["body"], "inline");
    assert_eq!(value["commit_id"], "abc");
    assert_eq!(value["event"], "COMMENT");
}

#[test]
fn parsers_read_login_reviews_and_the_created_id() {
    let login = login_from(br#"{"login":"octo"}"#).unwrap();
    assert_eq!(login, "octo");
    let listed =
        reviews_from(br#"[{"id":9,"user":{"login":"octo"},"state":"COMMENTED","body":"hi"}]"#)
            .unwrap();
    assert_eq!(listed[0].id, "9");
    assert_eq!(listed[0].author, "octo");
    assert_eq!(
        created_from(br#"{"id":6026927509}"#).unwrap().id,
        "6026927509"
    );
}

#[test]
fn deleted_reviewers_do_not_hide_eligible_reviews_on_later_pages() {
    let pages = reviews_from(
        br#"[{"id":1,"user":null,"state":"COMMENTED","body":"<!-- diffmate:publication=p1 -->"}]
[{"id":6026927509,"user":{"login":"octo"},"state":"COMMENTED","body":"<!-- diffmate:publication=p1 -->"},
 {"id":3,"user":{"login":"octo"},"state":"PENDING","body":""}]"#,
    )
    .unwrap();
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].id, "6026927509");
    assert_eq!(pages[0].author, "octo");
    assert_eq!(pages[0].body, "<!-- diffmate:publication=p1 -->");
    assert_eq!(pages[1].state, "PENDING");
}

#[test]
fn ambiguous_or_malformed_review_authors_are_still_errors() {
    for row in [
        r#"{"id":1,"user":null,"state":"PENDING","body":""}"#,
        r#"{"id":1,"user":null,"state":"","body":""}"#,
        r#"{"id":1,"state":"COMMENTED","body":""}"#,
        r#"{"id":1,"user":{"login":""},"state":"COMMENTED","body":""}"#,
        r#"{"id":1,"user":{},"state":"COMMENTED","body":""}"#,
    ] {
        assert!(
            reviews_from(format!("[{row}]").as_bytes()).is_err(),
            "{row}"
        );
    }
}
