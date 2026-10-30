use std::path::Path;

#[derive(Debug)]
pub struct GhCall {
    repositories: Vec<Option<String>>,
}

impl GhCall {
    const PROGRAM: &'static str = "gh";
    const FLAGS: &'static [&'static str] = &["-R", "--repo"];
    const JOINED: &'static [&'static str] = &["--repo=", "-R=", "-R"];
    const ENVIRONMENT: &'static str = "GH_REPO=";
    const URL: &'static str = "https://github.com/";
    const UNREADABLE: &'static [char] = &['$', '`', '(', ')', '{', '*', '?', '\\'];
    const URL_PATH_LEN: usize = 2;

    pub fn parse(segment: &[String]) -> Option<Self> {
        let start =
            segment.iter().position(|word| Path::new(word).file_name().is_some_and(|name| name == Self::PROGRAM))?;
        let before = segment.get(..start)?;
        let after = segment.get(start + 1..)?;
        let environment = before.iter().filter_map(|word| word.strip_prefix(Self::ENVIRONMENT)).map(Self::name);
        let flagged = after.iter().enumerate().flat_map(|(index, word)| Self::flagged(word, after.get(index + 1)));
        let urls = after.iter().filter(|word| word.starts_with(Self::URL)).map(|word| Self::name(word));
        Some(Self { repositories: environment.chain(flagged).chain(urls).collect() })
    }

    pub fn repositories_in(words: &[String]) -> Option<Vec<Option<String>>> {
        Self::parse(words).map(|call| call.repositories)
    }

    fn flagged(word: &str, next: Option<&String>) -> Vec<Option<String>> {
        let joined = Self::JOINED.iter().find_map(|prefix| word.strip_prefix(prefix));
        if Self::FLAGS.contains(&word) {
            vec![next.and_then(|value| Self::name(value))]
        } else {
            joined.map(Self::name).into_iter().collect()
        }
    }

    fn name(value: &str) -> Option<String> {
        let path = value.strip_prefix(Self::URL).map_or_else(
            || value.to_owned(),
            |rest| rest.split('/').take(Self::URL_PATH_LEN).collect::<Vec<_>>().join("/"),
        );
        (path.contains('/') && !path.contains(Self::UNREADABLE) && !path.starts_with('/') && !path.ends_with('/'))
            .then_some(path)
    }
}
