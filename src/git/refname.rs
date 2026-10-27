use crate::error::Error;

#[cfg(test)]
mod tests;

const FORBIDDEN: &str = ":?*[\\~^";

fn bad_char(c: char) -> bool {
    c.is_control() || c.is_whitespace() || FORBIDDEN.contains(c)
}

fn bad_part(part: &str) -> bool {
    part.is_empty() || part.starts_with('.') || part.to_ascii_lowercase().ends_with(".lock")
}

pub(super) fn validate_branch_name(name: &str) -> Result<(), Error> {
    let unsafe_name = name.starts_with('-')
        || name.ends_with('.')
        || name == "@"
        || name.contains("..")
        || name.contains("@{")
        || name.chars().any(bad_char)
        || name.split('/').any(bad_part);
    if unsafe_name {
        return Err(Error::UnsafeBranch(name.to_string()));
    }
    Ok(())
}

pub(super) fn nested_refs(a: &str, b: &str) -> bool {
    let inside = |outer: &str, inner: &str| {
        inner
            .strip_prefix(outer)
            .is_some_and(|rest| rest.starts_with('/'))
    };
    inside(a, b) || inside(b, a)
}
