use strum::EnumString;

use crate::env_policy::EnvPolicy;
use crate::tool::Tool;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum GitArgv {
    Var,
    ShowToplevel,
    VerifyRef,
    MergeBase,
    LsTree,
    CatBlob,
    ChangedNames,
    UntrackedNames,
    LogEmails,
    Porcelain,
    PushedHead,
    Head,
    PushedTips,
    Fetch,
    CheckIgnore,
    AbsoluteGitDir,
    WorktreeList,
    WorktreePrune,
    WorktreeConfig,
    BranchNoTrack,
    WorktreeAdd,
    ResetHard,
    WorktreeRemove,
    BranchDelete,
}

impl Tool for GitArgv {
    const PROGRAM: &'static str = "git";
    const POLICY: EnvPolicy = EnvPolicy::NoRepoVars;

    fn words(self) -> &'static [&'static str] {
        match self {
            Self::Var => &["var"],
            Self::ShowToplevel => &["rev-parse", "--show-toplevel"],
            Self::VerifyRef => &["rev-parse", "--verify", "--quiet"],
            Self::MergeBase => &["merge-base", "HEAD"],
            Self::LsTree => &["ls-tree", "-z", "--name-only"],
            Self::CatBlob => &["cat-file", "blob"],
            Self::ChangedNames => &["diff", "--name-only", "--no-renames", "--relative", "-z"],
            Self::UntrackedNames => &["ls-files", "--others", "--exclude-standard", "-z"],
            Self::LogEmails => &["log", "--format=%ae%n%ce", "HEAD", "--not"],
            Self::Porcelain => &["status", "--porcelain", "--untracked-files=all"],
            Self::PushedHead => &["branch", "--remotes", "--contains", "HEAD"],
            Self::Head => &["rev-parse", "HEAD"],
            Self::PushedTips => &["branch", "--remotes", "--format=%(refname:lstrip=3)", "--points-at"],
            Self::Fetch => &["fetch", "--quiet"],
            Self::CheckIgnore => &["check-ignore", "--quiet"],
            Self::AbsoluteGitDir => &["rev-parse", "--absolute-git-dir"],
            Self::WorktreeList => &["worktree", "list", "--porcelain"],
            Self::WorktreePrune => &["worktree", "prune"],
            Self::WorktreeConfig => &["config", "extensions.worktreeConfig", "true"],
            Self::BranchNoTrack => &["branch", "--no-track"],
            Self::WorktreeAdd => &["worktree", "add", "--quiet", "--no-checkout"],
            Self::ResetHard => &["reset", "--hard", "--quiet"],
            Self::WorktreeRemove => &["worktree", "remove", "--force"],
            Self::BranchDelete => &["branch", "-D"],
        }
    }
}
