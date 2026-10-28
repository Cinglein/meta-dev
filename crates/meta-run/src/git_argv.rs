use strum::EnumString;

use crate::env_policy::EnvPolicy;
use crate::tool::Tool;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum GitArgv {
    Var,
    ShowToplevel,
    VerifyRef,
}

impl Tool for GitArgv {
    const PROGRAM: &'static str = "git";
    const POLICY: EnvPolicy = EnvPolicy::NoRepoVars;

    fn words(self) -> &'static [&'static str] {
        match self {
            Self::Var => &["var"],
            Self::ShowToplevel => &["rev-parse", "--show-toplevel"],
            Self::VerifyRef => &["rev-parse", "--verify", "--quiet"],
        }
    }
}
