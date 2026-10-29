use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::process::Command;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Environment(BTreeMap<OsString, Option<OsString>>);

impl Environment {
    #[must_use]
    pub fn current() -> Self {
        Self(std::env::vars_os().map(|(name, value)| (name, Some(value))).collect())
    }

    pub(crate) fn without<'name>(mut self, names: impl IntoIterator<Item = &'name str>) -> Self {
        names.into_iter().for_each(|unwanted| drop(self.0.remove(OsStr::new(unwanted))));
        self
    }

    pub(crate) fn without_prefix(mut self, prefix: &str, kept: &[&str]) -> Self {
        self.0.retain(|name, _| {
            !name.as_encoded_bytes().starts_with(prefix.as_bytes()) || kept.iter().any(|kept| name == *kept)
        });
        self
    }

    pub(crate) fn only(mut self, names: &[&str]) -> Self {
        self.0.retain(|name, _| names.iter().any(|kept| name == *kept));
        self
    }

    pub(crate) fn dropped(self, kept: &Self, command: &mut Command) {
        let dropped = self.0.into_keys().filter(|name| !kept.0.contains_key(name));
        dropped.fold(command, |command, name| command.env_remove(name));
    }
}

impl From<BTreeMap<OsString, Option<OsString>>> for Environment {
    fn from(vars: BTreeMap<OsString, Option<OsString>>) -> Self {
        Self(vars)
    }
}

impl From<&Command> for Environment {
    fn from(command: &Command) -> Self {
        Self(command.get_envs().map(|(name, value)| (name.into(), value.map(OsStr::to_os_string))).collect())
    }
}
