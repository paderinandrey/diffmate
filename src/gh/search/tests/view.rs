use crate::gh::meta;

#[test]
fn pr_view_without_is_draft_is_false_and_true_stays_true() {
    let absent = meta::parse(view("").as_bytes()).unwrap();
    let present = meta::parse(view(r#","isDraft":true"#).as_bytes()).unwrap();
    assert!(!absent.is_draft);
    assert!(present.is_draft);
}

fn view(extra: &str) -> String {
    format!(
        r#"{{"number":1,"url":"https://github.com/acme/repo/pull/1","title":"t","body":"b","author":{{"login":"a"}},"baseRefName":"main","headRefName":"feature","headRefOid":"abc","isCrossRepository":false,"state":"OPEN"{extra}}}"#
    )
}
