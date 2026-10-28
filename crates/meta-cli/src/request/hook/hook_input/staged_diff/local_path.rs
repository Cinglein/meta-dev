use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum LocalPath {
    Scratch,
    Home,
}

impl LocalPath {
    pub fn in_line(_line: &str) -> Option<Self> {
        [Self::Scratch, Self::Home].into_iter().find(|_| false)
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
