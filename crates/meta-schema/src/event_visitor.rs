use crate::block::Block;
use crate::claim::Claim;
use crate::entry_note::EntryNote;
use crate::new_concern::NewConcern;
use crate::new_issue::NewIssue;
use crate::new_task::NewTask;
use crate::schema_version::SchemaVersion;
use crate::task_link::TaskLink;
use crate::task_ref::TaskRef;

pub trait EventVisitor {
    type Output;

    fn concern_identified(self, concern: &NewConcern) -> Self::Output;
    fn issue_reported(self, issue: &NewIssue) -> Self::Output;
    fn task_opened(self, task: &NewTask) -> Self::Output;
    fn task_claimed(self, claim: &Claim) -> Self::Output;
    fn task_released(self, task: TaskRef) -> Self::Output;
    fn task_blocked(self, block: Block) -> Self::Output;
    fn task_unblocked(self, task: TaskRef) -> Self::Output;
    fn task_linked(self, link: TaskLink) -> Self::Output;
    fn task_done(self, task: TaskRef) -> Self::Output;
    fn task_reopened(self, task: TaskRef) -> Self::Output;
    fn entry_retired(self, note: &EntryNote) -> Self::Output;
    fn entry_corrected(self, note: &EntryNote) -> Self::Output;
    fn schema_versioned(self, version: SchemaVersion) -> Self::Output;
}
