use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::process::Command;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Environment(BTreeMap<OsString, OsString>);

impl Environment {
    #[must_use]
    pub fn current() -> Self {
        Self(std::env::vars_os().collect())
    }

    pub(crate) fn without<'name>(mut self, names: impl IntoIterator<Item = &'name str>) -> Self {
        names.into_iter().for_each(|unwanted| drop(self.0.remove(OsStr::new(unwanted))));
        self
    }

    pub(crate) fn without_prefix(mut self, prefix: &str) -> Self {
        self.0.retain(|name, _| !name.as_encoded_bytes().starts_with(prefix.as_bytes()));
        self
    }

    pub(crate) fn only(mut self, names: &[&str]) -> Self {
        self.0.retain(|name, _| names.iter().any(|kept| name == *kept));
        self
    }

    pub(crate) fn exported(self, command: &mut Command) {
        command.env_clear().envs(self.0);
    }
}

impl From<BTreeMap<OsString, OsString>> for Environment {
    fn from(vars: BTreeMap<OsString, OsString>) -> Self {
        Self(vars)
    }
}

impl From<&Command> for Environment {
    fn from(command: &Command) -> Self {
        let set = command.get_envs().filter_map(|(name, value)| value.map(|value| (name.into(), value.into())));
        Self(set.collect())
    }
}
