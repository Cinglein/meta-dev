use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdminFile {
    GitDir,
    CommonDir,
    WorktreeConfig,
    Alternates,
}

impl FromStr for AdminFile {
    type Err = ();

    fn from_str(name: &str) -> Result<AdminFile, ()> {
        match name {
            "gitdir" => Ok(AdminFile::GitDir),
            "commondir" => Ok(AdminFile::CommonDir),
            "config.worktree" => Ok(AdminFile::WorktreeConfig),
            "alternates" => Ok(AdminFile::Alternates),
            _ => Err(()),
        }
    }
}

impl AdminFile {
    pub(crate) fn name(self) -> &'static str {
        match self {
            AdminFile::GitDir => "gitdir",
            AdminFile::CommonDir => "commondir",
            AdminFile::WorktreeConfig => "config.worktree",
            AdminFile::Alternates => "alternates",
        }
    }
}

impl fmt::Display for AdminFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::AdminFile;

    const NAMES: [&str; 4] = ["gitdir", "commondir", "config.worktree", "alternates"];
    const REFS: [&str; 3] = ["HEAD", "packed-refs", "MERGE_HEAD"];
    const INIT: [&str; 2] = ["init", "-q"];
    const REFS_DIR: &str = ".git/refs";
    const SHOUTED: &str = ".GIT";
    const LAID: &str = "refs/heads/laid";
    const EMPTY: &str = "";

    #[test]
    fn only_the_four_admin_files_parse_and_each_shows_its_own_name() {
        let shown = NAMES.map(|name| name.parse::<AdminFile>().map(|admin| admin.to_string()));
        assert!(shown.iter().zip(NAMES).all(|(shown, name)| shown.as_deref() == Ok(name)));
        assert!(REFS.iter().all(|name| name.parse::<AdminFile>().is_err()));
    }

    #[test]
    fn an_admin_file_is_read_only_from_a_git_admin_dir_never_from_refs_or_a_case_folded_git_dir() {
        let repository = tempfile::tempdir().unwrap();
        let init = std::process::Command::new("git").current_dir(repository.path()).args(INIT).status();
        assert!(init.unwrap().success());
        let refs = repository.path().join(REFS_DIR);
        std::fs::write(refs.join(AdminFile::GitDir.name()), EMPTY).unwrap();
        let shouted = repository.path().join(SHOUTED);
        let folded = shouted.exists();
        let refused = crate::Door::changed(shouted.join(LAID), crate::Change::DirAll).is_err();
        assert!(crate::Door::inspected(&refs, AdminFile::GitDir).is_err() && refused == folded);
    }
}
