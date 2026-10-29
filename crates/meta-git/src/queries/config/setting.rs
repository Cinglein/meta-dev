use crate::git_failure::GitFailure;
use crate::query::Query;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Setting(String);

impl From<String> for Setting {
    fn from(key: String) -> Setting {
        Setting(key)
    }
}

impl Query for Setting {
    type View = Option<String>;

    fn answered(&self, repository: &git2::Repository) -> Result<Option<String>, GitFailure> {
        Ok(repository.config()?.get_string(&self.0).ok())
    }
}
