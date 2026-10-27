use std::fmt;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Copy, Debug)]
pub struct SpawnedWords<'command>(&'command Command);

impl<'command> SpawnedWords<'command> {
    const SHOWN_LEN: usize = 2;
    const OPTION: char = '-';
    const NETWORK_PROGRAMS: [&'static str; 1] = ["gh"];
    const GIT: &'static str = "git";
    const NETWORK_GIT: [&'static str; 5] = ["fetch", "push", "pull", "ls-remote", "clone"];

    pub fn new(command: &'command Command) -> Self {
        Self(command)
    }

    pub fn networked(self) -> bool {
        let program = Path::new(self.0.get_program()).file_name().unwrap_or_default();
        let git_network = || self.plain().next().is_some_and(|word| Self::NETWORK_GIT.contains(&word));
        Self::NETWORK_PROGRAMS.iter().any(|name| program == *name) || (program == Self::GIT && git_network())
    }

    fn plain(self) -> impl Iterator<Item = &'command str> {
        self.0.get_args().filter_map(|word| word.to_str()).filter(|word| {
            !word.starts_with(Self::OPTION)
                && !word.is_empty()
                && word.chars().all(|letter| letter.is_ascii_lowercase() || letter == Self::OPTION)
        })
    }
}

impl fmt::Display for SpawnedWords<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shown: Vec<&str> = self.plain().take(Self::SHOWN_LEN).collect();
        formatter.write_str(&shown.join(" "))
    }
}
