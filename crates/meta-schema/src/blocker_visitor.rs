use crate::entity::{Issue, Task};
use crate::external_kind::ExternalKind;
use crate::id::Id;

pub trait BlockerVisitor {
    type Output;

    fn discussion(self, issue: Id<Issue>) -> Self::Output;
    fn task(self, task: Id<Task>) -> Self::Output;
    fn external(self, kind: ExternalKind) -> Self::Output;
}
