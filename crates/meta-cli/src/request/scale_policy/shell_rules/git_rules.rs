use serde::Deserialize;

use super::flag::Flag;
use super::refused::Refused;
use super::table::Table;
use super::union::Union;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitRules {
    #[serde(default)]
    forbidden: Vec<String>,
    #[serde(default)]
    nested: Table<Flag>,
}

impl GitRules {
    pub fn breach(&self, subcommand: &str) -> Option<Refused> {
        self.forbidden.iter().any(|word| word == subcommand).then(|| Refused::spelled(&["git", subcommand]))
    }

    pub fn nested(&self, subcommand: &str, args: &[String]) -> Option<String> {
        Flag::command_after(self.nested.words_for(subcommand), args)
    }
}

impl Union for GitRules {
    fn union(self, other: Self) -> Self {
        Self { forbidden: [self.forbidden, other.forbidden].concat(), nested: self.nested.union(other.nested) }
    }
}
