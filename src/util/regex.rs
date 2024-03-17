use anyhow::anyhow;
use regex::Captures;

pub fn regex_extract_match_group<T: From<String>>(
    c: &Captures,
    group: usize,
    key: &str,
) -> anyhow::Result<T> {
    let matched = c.get(group).ok_or(anyhow!("{} not found", key))?;
    Ok(T::from(matched.as_str().to_string()))
}
