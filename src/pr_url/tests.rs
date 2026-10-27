use super::{PrRef, parse};
use proptest::prelude::*;

const SUFFIXES: [&str; 8] = [
    "",
    "/",
    "/files",
    "/files/",
    "/checks",
    "/commits",
    "/commits/abc",
    "/commits/abc/",
];

fn parsed(url: &str) -> Option<PrRef> {
    parse(url).ok()
}

proptest! {
    #[test]
    fn valid_urls_parse_to_lower_cased_parts(
        owner in "[A-Za-z0-9_][A-Za-z0-9_.-]{0,20}",
        repo in "[A-Za-z0-9_][A-Za-z0-9_.-]{0,20}",
        number in 1_u64..=9_999_999_999,
        suffix in 0..SUFFIXES.len(),
        query in prop::sample::select(vec!["", "?diff=split", "?a=1&b=2"]),
        fragment in prop::sample::select(vec!["", "#discussion_r1"]),
    ) {
        let url = format!(
            "https://github.com/{owner}/{repo}/pull/{number}{}{query}{fragment}",
            SUFFIXES[suffix]
        );
        let expected = PrRef {
            owner: owner.to_ascii_lowercase(),
            repo: repo.to_ascii_lowercase(),
            number,
        };
        prop_assert_eq!(parsed(&url), Some(expected));
    }

    #[test]
    fn arbitrary_input_never_yields_an_unsafe_name(input in ".*") {
        if let Some(pr) = parsed(&input) {
            let safe = |name: &str| {
                let chars = name.chars().all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-'));
                chars && !name.is_empty() && name != "." && name != ".." && !name.starts_with('-')
            };
            prop_assert!(safe(&pr.owner) && safe(&pr.repo) && pr.number > 0);
        }
    }
}

#[test]
fn rejects_urls_outside_the_grammar() {
    let bad = [
        "http://github.com/o/r/pull/1",
        "https://example.com/o/r/pull/1",
        "https://github.com.evil.com/o/r/pull/1",
        "https://github.com/-o/r/pull/1",
        "https://github.com//r/pull/1",
        "https://github.com/o//pull/1",
        "https://github.com/./r/pull/1",
        "https://github.com/o/../pull/1",
        "https://github.com/o/r/pull/0",
        "https://github.com/o/r/pull/x1",
        "https://github.com/o/r/pull/12345678901",
        "https://github.com/o/r/issues/1",
        "https://github.com/o/r/pull/1/other",
        "https://github.com/o r/r/pull/1",
        "",
    ];
    for url in bad {
        assert_eq!(parsed(url), None, "{url}");
    }
}

#[test]
fn builds_urls_and_label() {
    let pr = parse("https://github.com/Owner/Repo/pull/7/files").unwrap();
    assert_eq!(pr.web_url(), "https://github.com/owner/repo/pull/7");
    assert_eq!(pr.remote_url(), "https://github.com/owner/repo.git");
    assert_eq!(pr.label(), "owner/repo#7");
}
