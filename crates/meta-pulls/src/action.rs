use std::ffi::OsString;
use std::fmt;

use meta_run::PullArgv;
use meta_schema::PullRequest;

use crate::live_pull::HeadCommit;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    CatchUp(PullRequest),
    Wait(PullRequest),
    Merge(PullRequest, HeadCommit),
}

impl Action {
    #[must_use]
    pub fn call(&self) -> Option<PullArgv> {
        match self {
            Self::CatchUp(_) => Some(PullArgv::UpdateBranch),
            Self::Merge(..) => Some(PullArgv::SquashMerge),
            Self::Wait(_) => None,
        }
    }

    #[must_use]
    pub fn operands(&self) -> Vec<OsString> {
        match self {
            Self::CatchUp(pull_request) | Self::Wait(pull_request) => vec![OsString::from(pull_request.to_string())],
            Self::Merge(pull_request, tested) => {
                vec![OsString::from(tested.as_ref()), OsString::from(pull_request.to_string())]
            }
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CatchUp(pull_request) => write!(formatter, "{pull_request} behind main: update branch"),
            Self::Wait(pull_request) => write!(formatter, "{pull_request} checks pending: wait"),
            Self::Merge(pull_request, tested) => {
                write!(formatter, "{pull_request} green and mergeable: squash-merge {tested}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use meta_run::{CommandLine, Port, PullArgv};

    use super::Action;
    use crate::live_pull::HeadCommit;

    const SHA: &str = "5a11c23b996afd573c618069841c1c52ada7ef7d";
    const ARGV: [&str; 7] = ["gh", "pr", "merge", "--squash", "--match-head-commit", SHA, "#187"];

    #[test]
    fn a_merge_squashes_only_the_head_its_checks_tested() {
        let action = Action::Merge("187".parse().unwrap(), HeadCommit::try_from(SHA.to_owned()).unwrap());
        let port = Port::<PullArgv>::new(Path::new(""));
        let argv = action.call().map(|call| CommandLine::from(&port.run(call, action.operands())));
        assert_eq!(argv.unwrap().words(), ARGV);
    }
}
