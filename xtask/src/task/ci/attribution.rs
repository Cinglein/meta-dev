use std::collections::BTreeSet;
use std::path::PathBuf;

use meta_golden::GoldenFailure;
use task::{FailureKind, FailureSet, JobName, Site};

use crate::task::failure::Failure;

pub struct Attribution;

impl Attribution {
    pub fn of(failure: &Failure, job: &JobName, collected: FailureSet) -> FailureSet {
        let found = Self::structured(failure).joined(collected);
        if found.is_empty() { found.with(FailureKind::Infra { job: job.clone() }) } else { found }
    }

    fn structured(failure: &Failure) -> FailureSet {
        match failure {
            Failure::Task(task::Failure::Found { found, .. }) => found.clone(),
            Failure::Task(task::Failure::Lint(reports)) => {
                let paths: BTreeSet<PathBuf> = reports
                    .iter()
                    .flat_map(task::Report::violations)
                    .filter_map(|violation| match violation.site() {
                        Site::File(path) | Site::Line(path, _) => Some(path.clone()),
                        Site::Workspace => None,
                    })
                    .collect();
                Self::with_paths(paths, |paths| FailureKind::Lint { paths })
            }
            Failure::Golden(GoldenFailure::Differs { path, .. }) => {
                Self::with_paths(BTreeSet::from([path.clone()]), |paths| FailureKind::Golden { paths })
            }
            Failure::Unreached { failure, .. } => Self::structured(failure),
            _ => FailureSet::default(),
        }
    }

    fn with_paths(paths: BTreeSet<PathBuf>, kind: impl FnOnce(Vec<PathBuf>) -> FailureKind) -> FailureSet {
        if paths.is_empty() {
            FailureSet::default()
        } else {
            FailureSet::default().with(kind(paths.into_iter().collect()))
        }
    }
}

#[cfg(test)]
mod tests {
    use task::{FailureKind, FailureSet, Report, Site, Violation};

    use super::Attribution;
    use crate::task::failure::Failure;

    const JOB: &str = "lint";
    const FILE: &str = "src/lib.rs";

    #[test]
    fn lint_paths_are_kept_and_a_failure_with_no_structure_is_infra() {
        let site = Site::File(FILE.into());
        let lint = Failure::Task(task::Failure::Lint(vec![Report::new("slip", vec![Violation::new(site, "a")])]));
        let bare = Failure::Task(task::Failure::Command(JOB.into()));
        let kinds = |failure: &Failure| Attribution::of(failure, &JOB.into(), FailureSet::default()).kinds().to_vec();
        let paths = vec![FILE.into()];
        assert_eq!(kinds(&lint), [FailureKind::Lint { paths }]);
        assert_eq!(kinds(&bare), [FailureKind::Infra { job: JOB.into() }]);
    }
}
