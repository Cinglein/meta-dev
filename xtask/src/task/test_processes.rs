mod test_key;
mod unmatched;

use git_trace::Spawns;
use task::{Amount, BaselineFile, Branch, BudgetId, Ledger, Measure, Policy, Selection, Workspace};
use test_time::ProcessCount;

use crate::task::failure::Failure;
use crate::task::ratchet::{Baseline, ViolationCount};
use test_key::TestKey;
pub use unmatched::Unmatched;

#[derive(Debug)]
pub struct TestProcesses;

impl TestProcesses {
    const MISSING: &str = "a test-processes budget with a baseline allowance in tooling/policy.toml";

    pub fn run(
        workspace: &Workspace,
        selection: &Selection,
        policy: &Policy,
        ledger: &mut Ledger,
    ) -> Result<(), Failure> {
        let budget = Self::allowance(policy)?;
        let file = Self::file(policy, budget)?;
        let spawns = ProcessCount::run(workspace, selection, policy, ledger)?;
        let allowed = Baseline::budgeted(workspace.root(), budget, file, policy.branch(Branch::Base))?;
        Self::within(&spawns, allowed, selection)?;
        ledger.spend(Measure::TestProcesses, Amount::Baseline(file.clone()));
        Ok(())
    }

    fn allowance(policy: &Policy) -> Result<&BudgetId, Failure> {
        policy.budgets().id_of(&Measure::TestProcesses).ok_or(Failure::Task(task::Failure::Unset(Self::MISSING)))
    }

    fn file<'policy>(policy: &'policy Policy, budget: &BudgetId) -> Result<&'policy BaselineFile, Failure> {
        let allowance = policy.budgets().budget(budget).and_then(|budget| budget.allowance().whole());
        allowance.and_then(Amount::baseline).ok_or(Failure::Task(task::Failure::Unset(Self::MISSING)))
    }

    fn within(spawns: &Spawns, allowed: Baseline, selection: &Selection) -> Result<(), Failure> {
        let observed: Baseline =
            spawns.per_test().into_iter().map(|(test, count)| (test, ViolationCount::from(count))).collect();
        let selected = allowed.retained(|key| key.parse::<TestKey>().is_ok_and(|key| key.selected(selection)));
        let drifts = observed.drifts(&selected);
        if drifts.is_empty() { Ok(()) } else { Err(Failure::Unmatched(Unmatched::new(drifts))) }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::ffi::OsStr;

    use git_trace::Spawns;
    use meta_run::SpawnTrace;
    use settings::EmbeddedDefaults;
    use task::{BaselineFile, Branch, BudgetId, Policy, Selection};

    use super::TestProcesses;
    use crate::task::fixture::Fixture;
    use crate::task::prior_policy::PriorPolicy;
    use crate::task::ratchet::Baseline;

    const NOTHING: &str = "";
    const COUNTED_POLICY: &str = "[budgets.counted]\nallowance = { baseline = \"a.toml\" }\n";
    const FILES: [&str; 2] = ["a.toml", "b.toml"];
    const BUDGETS: [&str; 2] = ["counted", "fresh"];
    const ONE: &str = "\"p it a\" = 1\n";
    const TWO: &str = "\"p it a\" = 2\n";
    const TEST: &str = "p it a";
    const PACKAGE: &str = "p";
    const UNSELECTED: &str = "\"p it a\" = 2\n\"q it z\" = 5\n";
    const ALLOWED: [&str; 3] = ["\"p it a\" = 2\n", "\"p it a\" = 3\n", "\"p it b\" = 2\n"];
    const DIFFER: [&str; 2] =
        ["p it a: 2 found, baseline 3", "p it a: 2 found, baseline 0\np it b: 0 found, baseline 2"];
    const BUDGET: &str = "[budgets.test-processes]\ndivision = \"performance-and-cost\"\nmeasure = \"test_processes\"\nallowance = { baseline = \"self/test-processes-baseline.toml\" }\nwindow = \"per_change\"\nsource = \"ci\"\n";

    #[test]
    fn a_selected_test_starts_exactly_its_allowance_and_one_not_listed_starts_none() {
        let root = tempfile::tempdir().unwrap();
        let dirs = [Spawns::GIT_TRACES, Spawns::SEALED_GIT_TRACES].map(|dir| root.path().join(TEST).join(dir));
        dirs.iter().chain([&root.path().join(Spawns::GIT_TRACES)]).try_for_each(std::fs::create_dir_all).unwrap();
        let trace = SpawnTrace::new(&root.path().join(TEST));
        trace.noted(OsStr::new(TEST)).and_then(|()| trace.noted(OsStr::new(TEST))).unwrap();
        let spawns = Spawns::traced(root.path()).unwrap();
        let (partial, whole) = (Selection::Crates(BTreeSet::from([PACKAGE.parse().unwrap()])), Selection::Workspace);
        let judged = |allowed: &str, selection: &Selection| {
            let allowed = toml::from_str::<Baseline>(allowed).unwrap();
            TestProcesses::within(&spawns, allowed, selection).map_err(|failure| failure.to_string()).err()
        };
        let ([exact, raised, unlisted], [fewer, missing]) = (ALLOWED.map(|allowed| judged(allowed, &whole)), DIFFER);
        let outside = judged(UNSELECTED, &partial).is_none() && judged(UNSELECTED, &whole).is_some();
        assert!(
            exact.is_none()
                && raised.is_some_and(|shown| shown.ends_with(fewer))
                && unlisted.is_some_and(|shown| shown.ends_with(missing))
                && outside
        );
    }

    #[test]
    fn a_policy_without_the_test_processes_budget_fails_closed() {
        let unbudgeted: Policy = toml::from_str(&Policy::DEFAULTS.replace(BUDGET, NOTHING)).unwrap();
        let file = |policy: &Policy| {
            TestProcesses::allowance(policy).and_then(|budget| TestProcesses::file(policy, budget)).is_ok()
        };
        assert!(file(&Policy::defaults()) && !file(&unbudgeted));
    }

    #[test]
    fn a_budgets_rows_are_kept_against_that_budgets_file_at_the_merge_base_and_a_new_budget_starts_free() {
        let fixture = Fixture::new();
        let ([kept, moved], [counted, fresh]) = (FILES, BUDGETS);
        fixture.write(PriorPolicy::PATH, COUNTED_POLICY).write(kept, ONE).branch();
        fixture.write(moved, TWO).committed();
        let budgeted = |budget: &str, file| {
            let (budget, file) = (BudgetId::from(budget.to_owned()), BaselineFile::from(file));
            Baseline::budgeted(fixture.root(), &budget, &file, Policy::defaults().branch(Branch::Base)).is_ok()
        };
        assert!(!budgeted(counted, moved) && budgeted(counted, kept) && budgeted(fresh, moved));
    }
}
