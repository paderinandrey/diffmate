use super::when;

pub(super) enum LiveHead {
    Fetched(String),
    Failed,
}

pub(super) fn describe(
    draft_head: &str,
    live: &LiveHead,
    known: Option<(&str, i64)>,
    now_ms: i64,
) -> String {
    let (head, checked) = match live {
        LiveHead::Fetched(head) => (head.as_str(), "just now".to_string()),
        LiveHead::Failed => match known {
            Some((head, checked_at)) => (
                head,
                format!(
                    "{} (live check failed)",
                    when::ago(now_ms.saturating_sub(checked_at))
                ),
            ),
            None => return "freshness unknown: head never checked (live check failed)".to_string(),
        },
    };
    let state = if head == draft_head {
        "current"
    } else {
        "stale"
    };
    let short: String = head.chars().take(12).collect();
    format!("{state}: head {short}, checked {checked}")
}

pub(crate) fn stored(draft_head: &str, known: Option<(&str, i64)>, now_ms: i64) -> String {
    let Some((head, checked_at)) = known else {
        return "freshness unknown: head never checked".to_string();
    };
    let state = if head == draft_head {
        "current"
    } else {
        "stale"
    };
    let short: String = head.chars().take(12).collect();
    let checked = when::ago(now_ms.saturating_sub(checked_at));
    format!("{state}: head {short}, checked {checked}")
}
