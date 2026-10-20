use thiserror::Error;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PrRef {
    pub owner: String,
    pub repo: String,
    pub number: u64,
}

#[derive(Debug, Error)]
#[error("not a github.com pull request URL")]
pub(crate) struct UrlError;

impl PrRef {
    pub fn web_url(&self) -> String {
        format!(
            "https://github.com/{}/{}/pull/{}",
            self.owner, self.repo, self.number
        )
    }

    pub fn remote_url(&self) -> String {
        format!("https://github.com/{}/{}.git", self.owner, self.repo)
    }

    pub fn label(&self) -> String {
        format!("{}/{}#{}", self.owner, self.repo, self.number)
    }
}

fn valid_name(name: &str) -> bool {
    let chars_ok = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'));
    let shape_ok = (1..=100).contains(&name.len()) && !name.starts_with('-');
    chars_ok && shape_ok && !matches!(name, "." | "..")
}

fn valid_tail(tail: &[&str]) -> bool {
    matches!(
        tail,
        [] | ["" | "files" | "checks" | "commits"]
            | ["files" | "checks" | "commits", ""]
            | ["commits", ..]
    )
}

pub(crate) fn parse(input: &str) -> Result<PrRef, UrlError> {
    let rest = input.strip_prefix("https://github.com/").ok_or(UrlError)?;
    let path = rest.split(['?', '#']).next().unwrap_or_default();
    let parts: Vec<&str> = path.split('/').collect();
    let [owner, repo, "pull", number, tail @ ..] = parts.as_slice() else {
        return Err(UrlError);
    };
    let digits = (1..=10).contains(&number.len()) && number.bytes().all(|b| b.is_ascii_digit());
    let number = number.parse::<u64>().ok().filter(|n| digits && *n > 0);
    match number {
        Some(number) if valid_name(owner) && valid_name(repo) && valid_tail(tail) => Ok(PrRef {
            owner: owner.to_ascii_lowercase(),
            repo: repo.to_ascii_lowercase(),
            number,
        }),
        _ => Err(UrlError),
    }
}
