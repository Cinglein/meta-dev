mod git_flag;

use git_flag::GitFlag;
use serde::Deserialize;

use super::git_breach::GitBreach;
use super::table::Table;
use super::union::Union;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitRules {
    #[serde(default)]
    forbidden: Vec<String>,
    #[serde(default)]
    flags: Table<GitFlag>,
}

impl GitRules {
    pub fn breach(&self, subcommand: &str, args: &[String]) -> Option<GitBreach> {
        let named = self.forbidden.iter().any(|word| word == subcommand).then(|| GitBreach::new(subcommand, None));
        let flagged = |arg: &String| self.flags.words_for(subcommand).iter().any(|flag| flag.matches(arg));
        named.or_else(|| args.iter().find(|arg| flagged(arg)).map(|arg| GitBreach::new(subcommand, Some(arg))))
    }
}

impl Union for GitRules {
    fn union(self, other: Self) -> Self {
        Self { forbidden: [self.forbidden, other.forbidden].concat(), flags: self.flags.union(other.flags) }
    }
}
