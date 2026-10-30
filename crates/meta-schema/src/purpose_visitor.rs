use crate::cause::Cause;
use crate::entity::Project;
use crate::id::Id;

pub trait PurposeVisitor {
    type Output;

    fn instruments(self, project: Id<Project>) -> Self::Output;
    fn serves(self, project: Id<Project>) -> Self::Output;
    fn caused_by(self, cause: Cause, project: Option<Id<Project>>) -> Self::Output;
}
