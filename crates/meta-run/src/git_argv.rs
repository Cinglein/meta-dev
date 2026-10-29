use strum::EnumString;

use crate::env_policy::EnvPolicy;
use crate::tool::Tool;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum GitArgv {
    Var,
    ShowToplevel,
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
    WorktreeList,
    ReflogWalk,
    RemoteHeads,
    Unpushed,
    IndependentTips,
    GitPath,
    RemoteUrl,
}

impl Tool for GitArgv {
    const PROGRAM: &'static str = "git";
    const POLICY: EnvPolicy = EnvPolicy::NoRepoVars;

    fn words(self) -> &'static [&'static str] {
        match self {
            Self::Var => &["var"],
            Self::ShowToplevel => &["rev-parse", "--show-toplevel"],
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
            Self::WorktreeList => &["worktree", "list", "--porcelain"],
            Self::ReflogWalk => &[
                "log",
                "--walk-reflogs",
                "--date=unix",
                "--decorate=full",
                "--decorate-refs=HEAD",
                "--decorate-refs=refs/heads/",
                "--format=%H%x1f%P%x1f%cn <%ce>%x1f%gn <%ge> %gd%x1f%D%x1f%gs",
                "HEAD",
                "--",
            ],
            Self::RemoteHeads => &["ls-remote", "--heads"],
            Self::Unpushed => &["rev-list", "--ignore-missing"],
            Self::IndependentTips => &["merge-base", "--independent"],
            Self::GitPath => &["rev-parse", "--git-path"],
            Self::RemoteUrl => &["remote", "get-url"],
        }
    }
}
