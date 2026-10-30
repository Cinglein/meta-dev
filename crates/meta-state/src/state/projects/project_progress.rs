use meta_schema::{Acceptance, Cause, Id, Project, PurposeVisitor, VerdictVisitor};

use crate::project_stage::ProjectStage;
use crate::refusal::Refusal;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectProgress {
    Proposed,
    Accepted(Acceptance),
    Instrumented(Acceptance),
    Delivered(Acceptance),
    Regressed(Acceptance),
}

impl ProjectProgress {
    pub fn stage(&self) -> ProjectStage {
        match self {
            Self::Proposed => ProjectStage::Proposed,
            Self::Accepted(_) => ProjectStage::Accepted,
            Self::Instrumented(_) => ProjectStage::Instrumented,
            Self::Delivered(_) => ProjectStage::Delivered,
            Self::Regressed(_) => ProjectStage::Regressed,
        }
    }

    pub fn accepted(&self, acceptance: &Acceptance) -> Result<Self, Refusal> {
        match self {
            Self::Proposed | Self::Accepted(_) | Self::Instrumented(_) => Ok(Self::Accepted(acceptance.clone())),
            Self::Delivered(_) | Self::Regressed(_) => Err(Refusal::ProjectIs(self.stage())),
        }
    }

    fn joined(&self) -> Result<(), Refusal> {
        match self {
            Self::Accepted(_) | Self::Instrumented(_) => Ok(()),
            Self::Proposed | Self::Delivered(_) | Self::Regressed(_) => Err(Refusal::Unattachable(self.stage())),
        }
    }
}

impl VerdictVisitor for &ProjectProgress {
    type Output = Result<ProjectProgress, Refusal>;

    fn instrumented(self) -> Self::Output {
        match self {
            ProjectProgress::Proposed => Err(Refusal::ProjectIs(ProjectStage::Proposed)),
            ProjectProgress::Accepted(acceptance) => Ok(ProjectProgress::Instrumented(acceptance.clone())),
            ProjectProgress::Instrumented(_) | ProjectProgress::Delivered(_) | ProjectProgress::Regressed(_) => {
                Err(Refusal::Unchanged)
            }
        }
    }

    fn uninstrumented(self) -> Self::Output {
        match self {
            ProjectProgress::Instrumented(acceptance) => Ok(ProjectProgress::Accepted(acceptance.clone())),
            ProjectProgress::Proposed
            | ProjectProgress::Accepted(_)
            | ProjectProgress::Delivered(_)
            | ProjectProgress::Regressed(_) => self.failing(),
        }
    }

    fn on_target(self) -> Self::Output {
        match self {
            ProjectProgress::Proposed => Err(Refusal::ProjectIs(ProjectStage::Proposed)),
            ProjectProgress::Instrumented(acceptance) | ProjectProgress::Regressed(acceptance) => {
                Ok(ProjectProgress::Delivered(acceptance.clone()))
            }
            ProjectProgress::Accepted(_) | ProjectProgress::Delivered(_) => Err(Refusal::Unchanged),
        }
    }

    fn failing(self) -> Self::Output {
        match self {
            ProjectProgress::Proposed => Err(Refusal::ProjectIs(ProjectStage::Proposed)),
            ProjectProgress::Delivered(acceptance) => Ok(ProjectProgress::Regressed(acceptance.clone())),
            ProjectProgress::Accepted(_) | ProjectProgress::Instrumented(_) | ProjectProgress::Regressed(_) => {
                Err(Refusal::Unchanged)
            }
        }
    }
}

impl PurposeVisitor for &ProjectProgress {
    type Output = Result<(), Refusal>;

    fn instruments(self, _: Id<Project>) -> Self::Output {
        self.joined()
    }

    fn serves(self, _: Id<Project>) -> Self::Output {
        self.joined()
    }

    fn caused_by(self, _: Cause, _: Option<Id<Project>>) -> Self::Output {
        match self {
            ProjectProgress::Proposed => Err(Refusal::Unattachable(ProjectStage::Proposed)),
            ProjectProgress::Accepted(_)
            | ProjectProgress::Instrumented(_)
            | ProjectProgress::Delivered(_)
            | ProjectProgress::Regressed(_) => Ok(()),
        }
    }
}
