pub(super) fn name(error: &toml::de::Error, text: &str, line: Option<usize>) -> Option<String> {
    let mut context = error.clone();
    context.set_input(None);
    if let Some(key) = context.to_string().lines().find_map(|line| {
        line.strip_prefix("in `")
            .and_then(|s| s.strip_suffix('`'))
            .map(str::to_owned)
    }) {
        return Some(key);
    }
    let source = text.split('\n').nth(line?.checked_sub(1)?)?;
    first_key(source).or_else(|| {
        source
            .match_indices('=')
            .find_map(|(index, _)| first_key(&format!("{} = 0", &source[..index])))
    })
}

fn first_key(text: &str) -> Option<String> {
    toml::from_str::<toml::Table>(text)
        .ok()?
        .into_iter()
        .next()
        .map(|(key, _)| key)
}
