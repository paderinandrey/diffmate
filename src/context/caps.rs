pub(crate) const PR_BODY_CAP: usize = 65_536;
pub(crate) const DIFF_CAP: usize = 1_048_576;

pub(crate) fn cap_split<'a>(
    name: &str,
    text: &'a str,
    total: usize,
    cap: usize,
) -> (Option<String>, &'a str) {
    if total <= cap {
        return (None, text);
    }
    let mut end = cap.min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    let head = &text[..end];
    let kept = head.rfind('\n').map_or(head, |at| &head[..=at]);
    let marker = format!("TRUNCATED: {name} kept {} of {total} bytes.", kept.len());
    (Some(marker), kept)
}

#[cfg_attr(not(test), expect(dead_code))]
pub(crate) fn cap_text(name: &str, text: &str, cap: usize) -> String {
    match cap_split(name, text, text.len(), cap) {
        (Some(marker), kept) => format!("{marker}\n{kept}"),
        (None, kept) => kept.to_string(),
    }
}

fn boundary(diff: &[u8], cap: usize) -> usize {
    let starts_line = |at: usize| at == 0 || diff[at - 1] == b'\n';
    let starts_unit = |at: usize| {
        let rest = &diff[at..];
        rest.starts_with(b"diff --git ") || rest.starts_with(b"@@ ")
    };
    (0..=cap)
        .rev()
        .find(|&at| starts_line(at) && starts_unit(at))
        .unwrap_or(0)
}

pub(crate) fn cap_diff(diff: &[u8], cap: usize) -> Vec<u8> {
    if diff.len() <= cap {
        return diff.to_vec();
    }
    let at = boundary(diff, cap);
    let total = diff.len();
    let marker =
        format!("TRUNCATED: diff.patch kept {at} of {total} bytes, cut at a hunk boundary.\n");
    [marker.as_bytes(), &diff[..at]].concat()
}
