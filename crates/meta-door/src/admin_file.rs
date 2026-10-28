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

    #[test]
    fn only_the_four_admin_files_parse_and_each_shows_its_own_name() {
        let shown = NAMES.map(|name| name.parse::<AdminFile>().map(|admin| admin.to_string()));
        assert!(shown.iter().zip(NAMES).all(|(shown, name)| shown.as_deref() == Ok(name)));
        assert!(REFS.iter().all(|name| name.parse::<AdminFile>().is_err()));
    }
}
