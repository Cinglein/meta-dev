use crate::environment::Environment;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvPolicy {
    Inherit,
    NoRepoVars,
    NoGitVars,
    Only(&'static [&'static str]),
}

impl EnvPolicy {
    const REPO_VARS: [&'static str; 3] = ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE"];
    const GIT_PREFIX: &'static str = "GIT_";
    const IDENTITY_VARS: [&'static str; 1] = ["EMAIL"];

    pub(crate) fn applied(self, inherited: Environment) -> Environment {
        match self {
            Self::Inherit => inherited,
            Self::NoRepoVars => inherited.without(Self::REPO_VARS),
            Self::NoGitVars => inherited.without_prefix(Self::GIT_PREFIX).without(Self::IDENTITY_VARS),
            Self::Only(kept) => inherited.only(kept),
        }
    }
}
