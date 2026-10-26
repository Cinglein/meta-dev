mod finding;
mod test_binary;
mod test_timing;

use std::cmp::Reverse;
use std::time::Duration;

use finding::Finding;
use test_binary::TestBinary;
use test_timing::TestTiming;

use task::Failure;
use task::Report;
use task::Selection;
use task::Site;
use task::Violation;
use task::Workspace;
use task::{Interval, Limit, Policy};

#[derive(Debug)]
pub struct TestTime {
    timings: Vec<TestTiming>,
}

impl TestTime {
    pub fn run(
        workspace: &Workspace,
        selection: &Selection,
        policy: &Policy,
    ) -> Result<(), Failure> {
        selection.reaching().map_or_else(
            || {
                println!("test time: no crate affected");
                Ok(())
            },
            |selection| {
                let timings = TestBinary::discover(workspace, selection, policy)?
                    .iter()
                    .map(TestBinary::timings)
                    .collect::<Result<Vec<Vec<TestTiming>>, Failure>>()?
                    .into_iter()
                    .flatten()
                    .collect();
                TestTime { timings }.report(policy).verdict()
            },
        )
    }

    fn report(mut self, policy: &Policy) -> Report {
        self.timings.sort_by_key(|timing| Reverse(timing.elapsed()));
        let total: Duration = self.timings.iter().map(TestTiming::elapsed).sum();
        println!(
            "test time: {} tests in {total:.2?}, slowest {}:",
            self.timings.len(),
            policy.limit(Limit::SlowestShown).count()
        );
        self.timings
            .iter()
            .take(policy.limit(Limit::SlowestShown).count())
            .for_each(|timing| println!("  {timing}"));
        let slow = self
            .timings
            .iter()
            .filter(|timing| timing.elapsed() > policy.interval(Interval::PerTest).duration())
            .map(|timing| {
                Violation::new(
                    Site::Workspace,
                    Finding::SlowTest {
                        timing: timing.clone(),
                        allowed: policy.interval(Interval::PerTest).duration(),
                    },
                )
            });
        let whole = (total > policy.interval(Interval::Total).duration()).then(|| {
            Violation::new(
                Site::Workspace,
                Finding::SlowSuite {
                    total,
                    allowed: policy.interval(Interval::Total).duration(),
                },
            )
        });
        let none = self
            .timings
            .is_empty()
            .then(|| Violation::new(Site::Workspace, Finding::NoTests));
        Report::new(
            "test time budget exceeded",
            slow.chain(whole).chain(none).collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{TestTime, TestTiming};
    use settings::EmbeddedDefaults;
    use std::path::Path;
    use std::time::Duration;
    use task::{Interval, Policy};
    const NAMES: [&str; 4] = ["target/debug/deps/bin-1a2b3c", "exact", "slow", "bulk"];
    const TICK: Duration = Duration::from_millis(1);
    const WHOLE_IN_UNITS: usize = 4;
    const REPORTED: [&str; 3] = ["bin bulk took", "bin slow took", "the suite took"];
    #[test]
    fn a_test_over_its_allowance_and_a_suite_over_its_total_are_reported_but_the_limits_pass() {
        let policy = Policy::defaults();
        let limit = policy.interval(Interval::PerTest).duration();
        let [binary, exact, slow, bulk] = NAMES;
        let timing = |name: &str, elapsed| TestTiming::new(Path::new(binary), name, elapsed);
        let exact = (0..WHOLE_IN_UNITS).map(|_| timing(exact, limit)).collect();
        let timings = vec![
            timing(slow, limit + TICK),
            timing(bulk, policy.interval(Interval::Total).duration()),
        ];
        let over = TestTime { timings }.report(&policy);
        assert!(
            TestTime { timings: exact }.report(&policy).is_clean() && over.lists_exactly(&REPORTED),
            "{over}"
        );
    }
}
