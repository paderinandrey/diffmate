use super::super::sanitize;
use proptest::prelude::*;

fn banned(c: char) -> bool {
    let control = c.is_control() && c != '\n' && c != '\t';
    let bidi = matches!(u32::from(c), 0x61C | 0x200E | 0x200F | 0x202A..=0x202E | 0x2066..=0x2069);
    control || bidi
}

fn text() -> impl Strategy<Value = String> {
    let low = (0u32..0xA0).prop_map(|c| char::from_u32(c).unwrap());
    let bidi = prop::sample::select(vec![
        '\u{202A}', '\u{202E}', '\u{2066}', '\u{2069}', '\u{200E}', '\u{200F}', '\u{61C}',
    ]);
    let chars = prop_oneof![any::<char>(), low, bidi];
    prop::collection::vec(chars, 0..40).prop_map(String::from_iter)
}

#[test]
fn escape_sequences_lose_their_escape() {
    assert_eq!(
        sanitize("\u{1b}[31mred\u{9b}x\u{85}\n\tend"),
        "[31mredx\n\tend"
    );
}

proptest! {
    #[test]
    fn nothing_that_drives_the_terminal_survives(text in text()) {
        let clean = sanitize(&text);
        prop_assert!(!clean.chars().any(banned));
        let kept: String = text.chars().filter(|c| !banned(*c)).collect();
        prop_assert_eq!(clean, kept);
    }
}
