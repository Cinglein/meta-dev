use std::cell::OnceCell;

use crate::request::project::Project;
use crate::request::remote_failure::RemoteFailure;
use crate::request::repository::Repository;

#[derive(Debug, Default)]
pub struct Remote {
    repository: OnceCell<Repository>,
}

impl Remote {
    pub fn resolved(&self, project: &Project) -> Result<&Repository, RemoteFailure> {
        self.repository.get().map_or_else(|| project.repository().map(|found| self.learned(found)), Ok)
    }

    fn learned(&self, found: Repository) -> &Repository {
        self.repository.get_or_init(|| found)
    }
}
