use strum::EnumString;

use crate::env_policy::EnvPolicy;
use crate::tool::Tool;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum MetaArgv {
    List,
}

impl Tool for MetaArgv {
    const PROGRAM: &'static str = "meta";
    const POLICY: EnvPolicy = EnvPolicy::Inherit;

    fn words(self) -> &'static [&'static str] {
        match self {
            Self::List => &["list"],
        }
    }
}
