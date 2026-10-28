use std::fmt;
use std::path::Path;

use crate::request::cli_failure::CliFailure;
use crate::request::local::local_file::LocalFile;

pub struct WorktreeIdentity<'text, Name: fmt::Display, Email: fmt::Display> {
    name: &'text Name,
    email: &'text Email,
}

impl<'text, Name: fmt::Display, Email: fmt::Display> WorktreeIdentity<'text, Name, Email> {
    const SECTIONS: [&'static str; 3] = ["user", "author", "committer"];
    const FILE: &'static str = "config.worktree";

    pub fn new(name: &'text Name, email: &'text Email) -> Self {
        Self { name, email }
    }

    pub fn written(&self, git_dir: &Path) -> Result<(), CliFailure> {
        LocalFile::new(git_dir.join(Self::FILE)).replaced(&self.to_string())
    }

    fn quoted(value: &impl fmt::Display) -> String {
        let escaped: String = value
            .to_string()
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

impl<Name: fmt::Display, Email: fmt::Display> fmt::Display for WorktreeIdentity<'_, Name, Email> {
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
        WorktreeIdentity::new(&AWKWARD, &EMAIL).written(directory.path()).unwrap();
        let listed = Command::new(GIT).args(LISTING).arg("--file").arg(&file).output().unwrap();
        let shown = String::from_utf8_lossy(&listed.stdout).into_owned();
        let lines = WorktreeIdentity::<&str, &str>::SECTIONS
            .iter()
            .flat_map(|key| [[key, NAME, AWKWARD, END].concat(), [key, MAIL, EMAIL, END].concat()]);
        assert!(lines.clone().all(|line| shown.contains(&line)), "{shown}");
    }
}
