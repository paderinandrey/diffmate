use crate::error::Error;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CommentLink {
    pub comment_id: String,
    pub thread_id: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ThreadState {
    pub thread_id: String,
    pub resolved: bool,
}

pub(super) fn comments(stdout: &[u8], review_id: &str) -> Result<Vec<CommentLink>, Error> {
    let wanted = review_id.parse::<i64>().map_err(|_| Error::Command {
        program: "gh",
        args: "api graphql".into(),
        detail: "invalid submitted review ID".into(),
    })?;
    let mut links = Vec::new();
    for thread in nodes(&pull(stdout)?, "reviewThreads") {
        let Some(thread_id) = thread["id"].as_str() else {
            continue;
        };
        for node in nodes(&thread, "comments") {
            if database_id(&node["pullRequestReview"]) != Some(wanted) {
                continue;
            }
            if let Some(link) = one(&node, thread_id) {
                links.push(link);
            }
        }
    }
    Ok(links)
}

pub(super) fn statuses(stdout: &[u8]) -> Result<Vec<ThreadState>, Error> {
    let mut rows = Vec::new();
    for node in nodes(&pull(stdout)?, "reviewThreads") {
        let Some(thread_id) = node["id"].as_str() else {
            continue;
        };
        let Some(resolved) = node["isResolved"].as_bool() else {
            continue;
        };
        rows.push(ThreadState {
            thread_id: thread_id.to_string(),
            resolved,
        });
    }
    Ok(rows)
}

fn database_id(node: &Value) -> Option<i64> {
    let id = &node["fullDatabaseId"];
    id.as_i64().or_else(|| id.as_str()?.parse().ok())
}

fn one(node: &Value, thread_id: &str) -> Option<CommentLink> {
    Some(CommentLink {
        comment_id: database_id(node)?.to_string(),
        thread_id: thread_id.to_string(),
        body: node["body"].as_str()?.to_string(),
    })
}

fn pull(stdout: &[u8]) -> Result<Value, Error> {
    let value: Value = serde_json::from_slice(stdout)?;
    value
        .pointer("/data/repository/pullRequest")
        .cloned()
        .ok_or_else(|| Error::Command {
            program: "gh",
            args: "api graphql".into(),
            detail: "the response holds no pull request".into(),
        })
}

pub(super) fn page_after(stdout: &[u8]) -> Result<Option<String>, Error> {
    let value: Value = serde_json::from_slice(stdout)?;
    let Some(page) = value.pointer("/data/repository/pullRequest/reviewThreads/pageInfo") else {
        return Ok(None);
    };
    if !page["hasNextPage"].as_bool().unwrap_or(false) {
        return Ok(None);
    }
    let cursor = page["endCursor"]
        .as_str()
        .filter(|cursor| !cursor.is_empty())
        .ok_or_else(|| Error::Command {
            program: "gh",
            args: "api graphql".into(),
            detail: "reviewThreads has another page but no endCursor".into(),
        })?;
    Ok(Some(cursor.to_string()))
}

fn nodes(value: &Value, field: &str) -> Vec<Value> {
    value[field]["nodes"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{comments, page_after, statuses};

    #[test]
    fn comments_link_only_the_submitted_review_using_full_database_ids() {
        use serde_json::json;

        for (review_id, comment_id, wanted, expected) in [
            (
                json!("6026927509"),
                json!("4669944031"),
                "6026927509",
                "4669944031",
            ),
            (
                json!(9_007_199_254_740_993_i64),
                json!(9_007_199_254_740_995_i64),
                "9007199254740993",
                "9007199254740995",
            ),
        ] {
            let thread = json!({"id":"t1","comments":{"nodes":[
                {"fullDatabaseId":comment_id,"body":"<!-- diffmate:finding=f1 -->","pullRequestReview":{"fullDatabaseId":review_id}},
                {"fullDatabaseId":8,"body":"other","pullRequestReview":{"fullDatabaseId":9_007_199_254_740_992_i64}}
            ]}});
            let response =
                json!({"data":{"repository":{"pullRequest":{"reviewThreads":{"nodes":[thread]}}}}});
            let links = comments(&serde_json::to_vec(&response).unwrap(), wanted).unwrap();
            assert_eq!(links.len(), 1);
            assert_eq!(links[0].comment_id, expected);
            assert_eq!(links[0].thread_id, "t1");
            assert_eq!(links[0].body, "<!-- diffmate:finding=f1 -->");
        }
    }

    #[test]
    fn statuses_read_is_resolved() {
        let raw = r#"{"data":{"repository":{"pullRequest":{"reviewThreads":{"nodes":[{"id":"t1","isResolved":false}]}}}}}"#;
        let rows = statuses(raw.as_bytes()).unwrap();
        assert_eq!(rows.len(), 1);
        assert!(!rows[0].resolved);
        assert_eq!(rows[0].thread_id, "t1");
    }

    #[test]
    fn a_missing_page_info_is_the_last_page() {
        let raw = r#"{"data":{"repository":{"pullRequest":{"reviewThreads":{"nodes":[]}}}}}"#;
        assert_eq!(page_after(raw.as_bytes()).unwrap(), None);
    }

    #[test]
    fn the_next_page_needs_an_end_cursor() {
        let raw = r#"{"data":{"repository":{"pullRequest":{"reviewThreads":{"pageInfo":{"hasNextPage":true}}}}}}"#;
        let error = page_after(raw.as_bytes()).unwrap_err();
        assert!(error.to_string().contains("no endCursor"), "{error}");
    }

    #[test]
    fn a_finished_page_returns_no_cursor() {
        let raw = r#"{"data":{"repository":{"pullRequest":{"reviewThreads":{"pageInfo":{"hasNextPage":false,"endCursor":null}}}}}}"#;
        assert_eq!(page_after(raw.as_bytes()).unwrap(), None);
        let raw = r#"{"data":{"repository":{"pullRequest":{"reviewThreads":{"pageInfo":{"hasNextPage":true,"endCursor":"abc"}}}}}}"#;
        assert_eq!(page_after(raw.as_bytes()).unwrap().as_deref(), Some("abc"));
    }
}
