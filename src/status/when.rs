pub(crate) fn ago(elapsed_ms: i64) -> String {
    match elapsed_ms {
        ..60_000 => "just now".to_string(),
        60_000..3_600_000 => format!("{}m ago", elapsed_ms / 60_000),
        3_600_000..86_400_000 => format!("{}h ago", elapsed_ms / 3_600_000),
        _ => format!("{}d ago", elapsed_ms / 86_400_000),
    }
}

#[expect(dead_code)]
pub(crate) fn until(remaining_ms: i64) -> String {
    match remaining_ms {
        ..=0 => "now".to_string(),
        1..60_000 => format!("in {}s", remaining_ms / 1_000),
        60_000..3_600_000 => format!("in {}m", remaining_ms / 60_000),
        _ => format!("in {}h", remaining_ms / 3_600_000),
    }
}
