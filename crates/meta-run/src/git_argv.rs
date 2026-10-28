use strum::EnumString;

use crate::env_policy::EnvPolicy;
use crate::tool::Tool;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum GitArgv {
    Var,
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
}

impl Tool for GitArgv {
    const PROGRAM: &'static str = "git";
    const POLICY: EnvPolicy = EnvPolicy::NoRepoVars;

    fn words(self) -> &'static [&'static str] {
        match self {
            Self::Var => &["var"],
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
        }
    }
}
