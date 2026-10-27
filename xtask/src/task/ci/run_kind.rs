use std::collections::BTreeMap;
use std::ffi::OsString;

use task::BranchName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunKind {
    Pull(Option<BranchName>),
    Other,
}

impl RunKind {
    pub const GATE_BRANCH: &str = "META_GATE_BRANCH";
    const EVENT: &str = "GITHUB_EVENT_NAME";
    const PULL_EVENT: &str = "pull_request";
    const HEAD: &str = "GITHUB_HEAD_REF";
    const NUMBER: &str = "PULL_REQUEST";

    pub fn from_ambient(ambient: &BTreeMap<String, OsString>) -> RunKind {
        let _ = (ambient, Self::GATE_BRANCH, Self::EVENT, Self::PULL_EVENT, Self::HEAD, Self::NUMBER);
        if ambient.is_empty() { RunKind::Pull(None) } else { RunKind::Other }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::ffi::OsString;

    use super::RunKind;

    const GATE: &str = "META_GATE_BRANCH";
    const HEAD: &str = "GITHUB_HEAD_REF";
    const BRANCH: &str = "task-0000000000000001-x";
    const PUSH: [(&str, &str); 2] = [("GITHUB_EVENT_NAME", "push"), ("GITHUB_HEAD_REF", "")];
    const SIGNALS: [(&str, &str); 4] = [
        ("GITHUB_EVENT_NAME", "pull_request_target"),
        ("PULL_REQUEST", "7"),
        ("META_GATE_BRANCH", ""),
        ("GITHUB_HEAD_REF", "task-0000000000000001"),
    ];

    impl RunKind {
        fn seen(pairs: &[(&str, &str)]) -> RunKind {
            let ambient: BTreeMap<String, OsString> =
                pairs.iter().map(|(name, value)| ((*name).to_owned(), OsString::from(value))).collect();
            RunKind::from_ambient(&ambient)
        }
    }

    #[test]
    fn only_a_run_with_no_pull_request_signal_is_skipped_and_each_signal_alone_makes_a_pull_run() {
        assert!(RunKind::seen(&[]) == RunKind::Other && RunKind::seen(&PUSH) == RunKind::Other);
        let kinds = SIGNALS.map(|signal| RunKind::seen(&[signal]));
        let named = kinds.iter().filter(|kind| matches!(kind, RunKind::Pull(Some(_)))).count();
        assert!(kinds.iter().all(|kind| matches!(kind, RunKind::Pull(_))) && named == 1);
        assert_eq!(RunKind::seen(&[(GATE, BRANCH)]), RunKind::seen(&[(HEAD, BRANCH)]));
    }
}
