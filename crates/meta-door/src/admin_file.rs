use std::fmt;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdminFile {
    GitDir,
    CommonDir,
    WorktreeConfig,
    Alternates,
    Locked,
    Reftable,
    Modules,
    RebaseMerge,
    RebaseApply,
    Sequencer,
}

impl AdminFile {
    const ALL: [AdminFile; 10] = [
        AdminFile::GitDir,
        AdminFile::CommonDir,
        AdminFile::WorktreeConfig,
        AdminFile::Alternates,
        AdminFile::Locked,
        AdminFile::Reftable,
        AdminFile::Modules,
        AdminFile::RebaseMerge,
        AdminFile::RebaseApply,
        AdminFile::Sequencer,
    ];

    #[must_use]
    pub fn within(path: &Path) -> Option<AdminFile> {
        Self::ALL.into_iter().find(|admin| path.ends_with(admin.name()))
    }

    #[must_use]
    pub fn directory_of(self, path: &Path) -> Option<&Path> {
        path.ancestors().nth(Path::new(self.name()).components().count())
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            AdminFile::GitDir => "gitdir",
            AdminFile::CommonDir => "commondir",
            AdminFile::WorktreeConfig => "config.worktree",
            AdminFile::Alternates => "objects/info/alternates",
            AdminFile::Locked => "locked",
            AdminFile::Reftable => "reftable",
            AdminFile::Modules => "modules",
            AdminFile::RebaseMerge => "rebase-merge",
            AdminFile::RebaseApply => "rebase-apply",
            AdminFile::Sequencer => "sequencer",
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
    use std::path::Path;

    use super::AdminFile;
    use crate::{Change, Door};

    const NAMED: [&str; 2] = ["/clone/.git/worktrees/x/config.worktree", "/clone/.git/objects/info/alternates"];
    const DIRS: [&str; 2] = ["/clone/.git/worktrees/x", "/clone/.git"];
    const REFS: [&str; 3] = ["/clone/.git/HEAD", "/clone/.git/packed-refs", "/clone/.git/MERGE_HEAD"];
    const INIT: [&str; 2] = ["init", "-q"];
    const REFS_DIR: &str = ".git/refs";
    const SHOUTED: &str = ".GIT";
    const ENTRY: &str = ".git";
    const LAID: &str = "refs/heads/laid";

    #[test]
    fn admin_files_are_named_by_their_tail_and_refs_never_are() {
        let found = NAMED.map(|path| AdminFile::within(Path::new(path)));
        let dirs =
            NAMED.iter().zip(found).map(|(path, admin)| admin.and_then(|admin| admin.directory_of(Path::new(path))));
        assert!(
            dirs.eq(DIRS.map(|dir| Some(Path::new(dir))))
                && found.iter().all(|admin| admin.is_some_and(|admin| admin.to_string().len() > 1))
        );
        assert!(REFS.iter().all(|path| AdminFile::within(Path::new(path)).is_none()));
    }

    #[test]
    fn an_admin_file_is_read_only_from_a_git_admin_dir_never_from_refs_or_a_case_folded_git_dir() {
        let repository = tempfile::tempdir().unwrap();
        let init = std::process::Command::new("git").current_dir(repository.path()).args(INIT).status();
        assert!(init.unwrap().success());
        let shouted = repository.path().join(SHOUTED);
        let folded = shouted.exists();
        let refused = Door::changed(shouted.join(LAID), Change::DirAll).is_err();
        let git_dir = repository.path().join(ENTRY);
        let admin = Door::inspected(&git_dir).is_ok();
        let refs = repository.path().join(REFS_DIR);
        let locked = |dir: &std::path::Path| Door::admin_kind(&dir.join(AdminFile::Locked.name()));
        let enclosed = locked(&git_dir).is_some() && locked(&refs).is_none();
        assert!(Door::inspected(&refs).is_err() && refused == folded && admin && enclosed);
    }
}
