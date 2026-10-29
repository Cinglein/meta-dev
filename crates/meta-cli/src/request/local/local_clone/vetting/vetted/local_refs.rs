use meta_run::Budget;

use super::link::{Link, RootRef, Scope};
use super::ref_line::RefLine;
use super::ref_list::RefList;
use crate::request::local::local_clone::vetting::asked::{Resolve, WorktreeRefs};
use crate::request::local::local_clone::vetting::cause::Cause;
use crate::request::local::local_clone::vetting::disk::Disk;
use crate::request::local::local_clone::vetting::entry_kind::EntryKind;
use crate::request::local::local_clone::vetting::kept::Kept;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalRefs {
    roots: Vec<String>,
    resolved: Vec<Option<String>>,
    listed: RefList,
}

impl LocalRefs {
    const HEAD: &'static str = "HEAD";
    const NO_HEAD: &'static str = "HEAD does not resolve";

    pub fn observed(link: &Link, branch: &str, budget: Budget) -> Result<Self, Kept> {
        let walked = Disk::walked(link.git_dir());
        let named = walked.iter().filter(|found| found.kind() == EntryKind::File);
        let roots: Vec<String> = named
            .filter_map(|found| found.path().to_str()?.parse::<RootRef>().ok())
            .map(|root| root.peeled())
            .collect();
        let wanted = [Self::HEAD.to_owned()].into_iter().chain(roots.iter().cloned()).collect();
        let repo = link.repo(Scope::Worktree, budget);
        let resolved = repo.observe(&Resolve::new(wanted))?;
        let listed = repo.observe(&WorktreeRefs::new(branch))?;
        Ok(Self { roots, resolved, listed })
    }

    #[cfg(test)]
    pub fn new(roots: Vec<String>, resolved: Vec<Option<String>>, listed: RefList) -> Self {
        Self { roots, resolved, listed }
    }

    pub fn refs(&self) -> Result<RefList, Kept> {
        let (head, objects) = self.resolved.split_first().ok_or_else(|| Kept::Uninspected(Cause::new(Self::NO_HEAD)))?;
        let head = head.as_deref().ok_or_else(|| Kept::Uninspected(Cause::new(Self::NO_HEAD)))?;
        let local = self.listed.clone().headed(RefLine::head(head));
        let roots = self
            .roots
            .iter()
            .zip(objects)
            .filter_map(|(peeled, object)| Some(RefLine::pseudo(peeled, object.as_deref()?)));
        Ok(roots.fold(local, RefList::headed))
    }
}
