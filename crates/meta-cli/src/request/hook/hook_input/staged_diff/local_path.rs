use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum LocalPath {
    Scratch,
    Home,
}

impl LocalPath {
    const SCRATCH: &'static str = concat!("/private/tmp/", "claude-");
    const HOME_ROOTS: [&'static str; 2] = ["/Users/", "/home/"];
    const NAME_SYMBOLS: &'static str = "._-";

    pub fn in_line(line: &str) -> Option<Self> {
        line.contains(Self::SCRATCH)
            .then_some(Self::Scratch)
            .or_else(|| Self::HOME_ROOTS.iter().find_map(|root| Self::home_under(line, root)))
    }

    fn home_under(line: &str, root: &str) -> Option<Self> {
        line.match_indices(root).find_map(|(at, _)| {
            let first = line.get(at + root.len()..)?.chars().next()?;
            (first.is_ascii_alphanumeric() || Self::NAME_SYMBOLS.contains(first)).then_some(Self::Home)
        })
    }
}

impl fmt::Display for LocalPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Scratch => "scratch path",
            Self::Home => "home path",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::LocalPath;

    const FOUND: [&str; 3] = [
        concat!("cd /private/tmp/", "claude-501/x"),
        concat!("x=/Users/", "alice/notes"),
        concat!("\"/home/", "alice/x\""),
    ];
    const BARE: [&str; 4] = ["/Users/", "/home/", "/Users/<name>/x", "see /Users/ and /private/tmp/x"];

    #[test]
    fn scratch_and_home_paths_are_found_anywhere_in_a_line_and_bare_roots_or_placeholders_are_not() {
        let found = FOUND.map(LocalPath::in_line);
        assert_eq!(found, [Some(LocalPath::Scratch), Some(LocalPath::Home), Some(LocalPath::Home)]);
        assert!(BARE.iter().all(|line| LocalPath::in_line(line).is_none()));
    }
}
