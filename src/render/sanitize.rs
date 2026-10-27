fn is_bidi_control(c: char) -> bool {
    matches!(
        c,
        '\u{61C}' | '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'
    )
}

pub(crate) fn sanitize(text: &str) -> String {
    text.chars()
        .filter(|c| matches!(c, '\n' | '\t') || !(c.is_control() || is_bidi_control(*c)))
        .collect()
}
