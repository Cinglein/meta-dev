use strum::EnumString;

use crate::env_policy::EnvPolicy;
use crate::tool::Tool;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum GitArgv {
    Var,
}

impl Tool for GitArgv {
    const PROGRAM: &'static str = "git";
    const POLICY: EnvPolicy = EnvPolicy::NoRepoVars;

    fn words(self) -> &'static [&'static str] {
        match self {
            Self::Var => &["var"],
        }
    }
}
