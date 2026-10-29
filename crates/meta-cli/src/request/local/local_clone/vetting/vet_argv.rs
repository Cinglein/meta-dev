use meta_run::{EnvPolicy, Tool};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VetArgv {
    WorktreeList,
    RemoteRefs,
    Index,
    Status,
    Beyond,
    Resolve,
    WorktreeRefs,
    PausedRoots,
    Move,
    Lock,
    Unlock,
    Remove,
    Unbranch,
}

impl Tool for VetArgv {
    const PROGRAM: &'static str = "git";
    const POLICY: EnvPolicy = EnvPolicy::NoGitVars;

    fn words(self) -> &'static [&'static str] {
        match self {
            Self::WorktreeList => &["worktree", "list", "--porcelain"],
            Self::RemoteRefs => &["ls-remote"],
            Self::Index => &["ls-files", "-s", "-v", "-z"],
            Self::Status => &[
                "status",
                "--porcelain",
                "-z",
                "--untracked-files=all",
                "--ignored=matching",
                "--ignore-submodules=none",
                "--no-renames",
            ],
            Self::Beyond => &["rev-list", "--max-count=1", "--ignore-missing"],
            Self::Resolve => &["cat-file", "--batch-check=%(objectname)"],
            Self::WorktreeRefs => {
                &["for-each-ref", "--format=%(objectname) %(refname)", "refs/worktree/", "refs/bisect/", "refs/rewritten/"]
            }
            Self::PausedRoots => &[
                "for-each-ref",
                "--include-root-refs",
                "--format=%(refname)",
                "CHERRY_PICK_HEAD",
                "REVERT_HEAD",
                "REBASE_HEAD",
            ],
            Self::Move => &["worktree", "move"],
            Self::Lock => &["worktree", "lock", "--reason", "meta gc quarantine"],
            Self::Unlock => &["worktree", "unlock"],
            Self::Remove => &["worktree", "remove"],
            Self::Unbranch => &["update-ref", "-d"],
        }
    }
}
