use std::fmt;

pub struct WorktreeIdentity<'text> {
    name: &'text str,
    email: &'text str,
}

impl<'text> WorktreeIdentity<'text> {
    const SECTIONS: [&'static str; 3] = ["user", "author", "committer"];

    pub fn new(name: &'text str, email: &'text str) -> Self {
        Self { name, email }
    }

    fn quoted(value: &str) -> String {
        let escaped: String = value
            .chars()
            .flat_map(|character| match character {
                '\\' => vec!['\\', '\\'],
                '"' => vec!['\\', '"'],
                '\n' => vec!['\\', 'n'],
                '\t' => vec!['\\', 't'],
                other => vec![other],
            })
            .collect();
        format!("\"{escaped}\"")
    }
}

impl fmt::Display for WorktreeIdentity<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Self::SECTIONS.iter().try_for_each(|section| {
            writeln!(
                formatter,
                "[{section}]\n\tname = {}\n\temail = {}",
                Self::quoted(self.name),
                Self::quoted(self.email)
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::WorktreeIdentity;

    const AWKWARD: &str = "a \"b\" \\ c\td # ;e";
    const EMAIL: &str = "1+agent@users.noreply.github.com";
    const GIT: &str = "git";
    const LISTING: [&str; 2] = ["config", "--list"];
    const FILE: &str = "config.worktree";
    const NAME: &str = ".name=";
    const MAIL: &str = ".email=";
    const END: &str = "\n";

    #[test]
    fn git_reads_back_every_identity_key_with_awkward_characters_intact() {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join(FILE);
        std::fs::write(&file, WorktreeIdentity::new(AWKWARD, EMAIL).to_string()).unwrap();
        let listed = Command::new(GIT).args(LISTING).arg("--file").arg(&file).output().unwrap();
        let shown = String::from_utf8_lossy(&listed.stdout).into_owned();
        let lines = WorktreeIdentity::SECTIONS
            .iter()
            .flat_map(|key| [[key, NAME, AWKWARD, END].concat(), [key, MAIL, EMAIL, END].concat()]);
        assert!(lines.clone().all(|line| shown.contains(&line)), "{shown}");
    }
}
