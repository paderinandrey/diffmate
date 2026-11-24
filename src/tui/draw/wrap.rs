use unicode_width::UnicodeWidthChar;

pub(super) fn lines(text: &str, width: usize) -> impl Iterator<Item = &str> {
    let width = width.max(1);
    text.split('\n').flat_map(move |line| {
        let mut remaining = Some(line);
        std::iter::from_fn(move || {
            let text = remaining.take()?;
            let mut cols = 0;
            let mut last_space = None;
            for (at, ch) in text.char_indices() {
                let size = ch.width().unwrap_or(0);
                if at > 0 && cols + size > width {
                    let split = if ch == ' ' {
                        at
                    } else {
                        last_space.unwrap_or(at)
                    };
                    remaining = Some(text[split..].trim_start_matches(' '));
                    if remaining == Some("") {
                        remaining = None;
                    }
                    return Some(&text[..split]);
                }
                if ch == ' ' && at > 0 && cols > 0 {
                    last_space = Some(at);
                }
                cols += size;
            }
            Some(text)
        })
    })
}
