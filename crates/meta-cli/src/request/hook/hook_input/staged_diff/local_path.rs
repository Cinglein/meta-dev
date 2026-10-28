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
            .or_else(|| Self::HOME_ROOTS.iter().any(|root| Self::names_a_home(line, root)).then_some(Self::Home))
    }

    fn names_a_home(line: &str, root: &str) -> bool {
        line.match_indices(root).any(|(at, _)| {
            line.get(at + root.len()..)
                .and_then(|after| after.chars().next())
                .is_some_and(|first| first.is_ascii_alphanumeric() || Self::NAME_SYMBOLS.contains(first))
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

    const SCRATCH: &str = concat!("/private/tmp/", "claude-501/scratch");
    const USERS: &str = "/Users/";
    const HOMES: &str = "/home/";
    const TAIL: &str = "alice/notes";
    const BARE: [&str; 4] = ["/Users/", "/home/", "/Users/<name>/x", "see /Users/ and /private/tmp/x"];

    #[test]
    fn scratch_and_home_paths_are_found_anywhere_in_a_line_and_bare_roots_or_placeholders_are_not() {
        let found = [format!("cd {SCRATCH}"), format!("x={USERS}{TAIL}"), format!("\"{HOMES}{TAIL}\"")]
            .map(|line| LocalPath::in_line(&line));
        assert_eq!(found, [Some(LocalPath::Scratch), Some(LocalPath::Home), Some(LocalPath::Home)]);
        assert!(BARE.iter().all(|line| LocalPath::in_line(line).is_none()));
    }
}
