use std::path::Path;

use affected::Changes;
use task::Policy;

use crate::task::Task;
use crate::task::ci::{LocalGate, Posting};
use crate::task::failure::Failure;

#[derive(Debug, PartialEq, Eq)]
pub struct Coverage {
    changes: Changes,
    posting: Posting,
}

impl Coverage {
    pub const EVERYTHING: Coverage = Coverage { changes: Changes::Everything, posting: Posting::Off };
    const AFFECTED_FLAG: &str = "--affected";
    const STATUS_FLAG: &str = "--status";
    const KNOWN: &str = "--affected, --status (ci only)";

    pub fn from_flags(task: Task, mut flags: impl Iterator<Item = String>) -> Result<Coverage, task::Failure> {
        flags.try_fold(Self::EVERYTHING, |coverage, flag| match flag.as_str() {
            Self::AFFECTED_FLAG => Ok(Coverage { changes: Changes::SinceMain, ..coverage }),
            Self::STATUS_FLAG if task == Task::Ci => Ok(Coverage { posting: Posting::Requested, ..coverage }),
            _ => Err(task::Failure::UnknownFlag { flag, known: Self::KNOWN }),
        })
    }

    pub fn changes(&self) -> Changes {
        self.changes
    }

    pub fn gate(&self, root: &Path, policy: &Policy) -> Result<Option<LocalGate>, Failure> {
        match self.posting {
            Posting::Requested => LocalGate::new(root, policy).map(Some),
            Posting::Off => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::Coverage;
    use crate::task::Task;
    use affected::Changes;
    use settings::EmbeddedDefaults;
    use task::{Failure, Policy};

    const AFFECTED: &str = "--affected";
    const STATUS: &str = "--status";
    const UNKNOWN: &str = "--all";

    impl Coverage {
        fn parsed(task: Task, flags: &[&str]) -> Result<Coverage, Failure> {
            Coverage::from_flags(task, flags.iter().map(ToString::to_string))
        }
    }

    #[test]
    fn no_flag_covers_everything_the_affected_flag_the_changes_and_others_are_refused() {
        assert_eq!(Coverage::parsed(Task::Ci, &[]).ok(), Some(Coverage::EVERYTHING));
        let affected = Coverage::parsed(Task::Test, &[AFFECTED]).map(|coverage| coverage.changes());
        assert_eq!(affected.ok(), Some(Changes::SinceMain));
        assert!(matches!(Coverage::parsed(Task::Ci, &[UNKNOWN]), Err(Failure::UnknownFlag { .. })));
    }

    #[test]
    fn only_ci_takes_the_status_flag_and_only_it_asks_for_a_gate() {
        let gate = |flags: &[&str]| {
            Coverage::parsed(Task::Ci, flags).ok().map(|coverage| coverage.gate(Path::new(""), &Policy::defaults()))
        };
        assert!(matches!(gate(&[STATUS, AFFECTED]), Some(Ok(Some(_)))));
        assert!(matches!(gate(&[AFFECTED]), Some(Ok(None))));
        assert!(matches!(Coverage::parsed(Task::Test, &[STATUS]), Err(Failure::UnknownFlag { .. })));
    }
}
