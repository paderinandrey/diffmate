fn unquote(text: &str) -> Option<String> {
    let inner = text.strip_prefix('"')?.strip_suffix('"')?;
    let mut bytes = Vec::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            let mut buf = [0; 4];
            bytes.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            continue;
        }
        let escaped = chars.next()?;
        let byte = match escaped {
            'a' => 7,
            'b' => 8,
            'f' => 12,
            'n' => b'\n',
            'r' => b'\r',
            't' => b'\t',
            'v' => 11,
            '0'..='3' => {
                let mut value = escaped.to_digit(8)?;
                for _ in 0..2 {
                    value = value * 8 + chars.next()?.to_digit(8)?;
                }
                u8::try_from(value).ok()?
            }
            other => u8::try_from(other).ok()?,
        };
        bytes.push(byte);
    }
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

pub(super) fn target(text: &str, prefix: char) -> Option<String> {
    let text = text.strip_suffix('\t').unwrap_or(text);
    let name = unquote(text).unwrap_or_else(|| text.to_string());
    if name == "/dev/null" {
        return None;
    }
    let stripped = name
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_prefix('/'));
    Some(stripped.map_or(name.clone(), str::to_string))
}
