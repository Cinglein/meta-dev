use affected::{Affected, Changes};
use task::Failure;
use task::Selection;
use task::Workspace;

pub struct Coverage;

impl Coverage {
    const AFFECTED_FLAG: &str = "--affected";

    pub fn from_flags(mut flags: impl Iterator<Item = String>, workspace: &Workspace) -> Result<Selection, Failure> {
        flags.try_fold(Selection::Workspace, |_, flag| {
            if flag == Self::AFFECTED_FLAG {
                Affected::selection(workspace, Changes::SinceMain)
            } else {
                Err(Failure::UnknownFlag { flag, known: Self::AFFECTED_FLAG })
            }
        })
    }
}
