use std::collections::BTreeMap;

use git_trace::Spawns;
use task::{Amount, Ledger, Measure, Policy, Selection, Workspace};
use test_time::GitCount;

use crate::task::failure::Failure;
use crate::task::ratchet::{Baseline, Drift, Ratchet, ViolationCount};

#[derive(Debug)]
pub struct TestProcesses;

impl TestProcesses {
    pub fn run(
        workspace: &Workspace,
        selection: &Selection,
        policy: &Policy,
        ledger: &mut Ledger,
    ) -> Result<(), Failure> {
        let spawns = GitCount::run(workspace, selection, policy, ledger)?;
        let budget = policy.budgets().whole(&Measure::TestProcesses);
        budget.and_then(|budget| budget.allowance().whole()).and_then(Amount::baseline).map_or(Ok(()), |file| {
            Self::within(&spawns, &Baseline::budgeted(workspace.root(), file, policy)?)?;
            ledger.spend(Measure::TestProcesses, Amount::Baseline(file.clone()));
            Ok(())
        })
    }

    fn within(spawns: &Spawns, allowed: &Baseline) -> Result<(), Failure> {
        let observed: Baseline =
            spawns.per_test().into_iter().map(|(test, count)| (test, ViolationCount::from(count))).collect();
        let grown: Vec<Drift> = observed.drifts(allowed).into_iter().filter(Drift::grew).collect();
        if grown.is_empty() { Ok(()) } else { Err(Failure::Ratchet(Ratchet::new(grown, BTreeMap::new()))) }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use git_trace::Spawns;

    use super::TestProcesses;
    use crate::task::ratchet::Baseline;

    const TRACES: &str = "it a/git-traces";
    const STARTS: (&str, &str) = ("it a/starts", "sh\nsh\n");
    const ALLOWED: [&str; 3] = ["\"it a\" = 2\n", "\"it a\" = 3\n", "\"it b\" = 2\n"];
    const GREW: &str = "it a: 2 found, baseline 0\n";

    #[test]
    fn a_test_may_start_up_to_its_allowance_and_one_not_listed_may_start_none() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join(TRACES))
            .and_then(|()| fs::write(root.path().join(STARTS.0), STARTS.1))
            .unwrap();
        let spawns = Spawns::traced(root.path()).unwrap();
        let judged = ALLOWED.map(|allowed| {
            let allowed = toml::from_str::<Baseline>(allowed).unwrap();
            TestProcesses::within(&spawns, &allowed).map_err(|failure| failure.to_string())
        });
        assert!(judged[0].is_ok() && judged[1].is_ok() && judged[2].as_ref().err().is_some_and(|shown| shown == GREW));
    }
}
