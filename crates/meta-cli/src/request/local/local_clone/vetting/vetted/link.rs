mod guard;
mod path_line;
mod pointers;
mod resolved;
mod root_ref;
mod scope;
mod step;
mod worktree_config;

use std::path::{Path, PathBuf};

use meta_run::Budget;

use crate::request::local::local_clone::vetting::context::Context;
use crate::request::local::local_clone::vetting::kept::Kept;
use crate::request::local::local_clone::vetting::repo::Repo;
use crate::request::local::local_clone::vetting::thaw::Thaw;
use crate::request::local::worktree_id::WorktreeId;
pub use guard::Guard;
pub use pointers::Pointers;
pub use resolved::Resolved;
pub use root_ref::RootRef;
pub use scope::Scope;
pub use step::Step;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    path: PathBuf,
    git_dir: PathBuf,
    common: PathBuf,
}

impl Link {
    pub fn new(path: &Path, git_dir: PathBuf, common: &Path) -> Self {
        Self { path: path.to_path_buf(), git_dir, common: common.to_path_buf() }
    }

    pub fn observed(path: &Path, id: Option<&WorktreeId>, context: &Context) -> Result<Self, Kept> {
        let link = Pointers::observed(path).proved(path, id, context.common())?;
        Guard::observed(&link, context.budget()).judged().map(|()| link)
    }

    pub fn repo(&self, scope: Scope, budget: Budget) -> Repo {
        match scope {
            Scope::Worktree => Repo::worktree(&self.git_dir, &self.path, budget),
            Scope::Common => Repo::common(&self.common, budget),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn git_dir(&self) -> &Path {
        &self.git_dir
    }

    pub fn common(&self) -> &Path {
        &self.common
    }

    pub fn relocated(&self, path: &Path) -> Self {
        Self { path: path.to_path_buf(), ..self.clone() }
    }

    pub fn thaw_plan(&self, origin: &Path) -> Thaw {
        Thaw::planned(origin, Thaw::ledger_of(&self.path, &self.common))
    }
}
