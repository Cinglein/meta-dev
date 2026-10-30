use crate::change::Change;
use crate::git_failure::GitFailure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigFlag(String);

impl From<String> for ConfigFlag {
    fn from(key: String) -> ConfigFlag {
        ConfigFlag(key)
    }
}

impl Change for ConfigFlag {
    type Applied = ();

    fn applied(&self, repository: &git2::Repository) -> Result<(), GitFailure> {
        repository.config()?.open_level(git2::ConfigLevel::Local)?.set_bool(&self.0, true)?;
        Ok(())
    }
}
