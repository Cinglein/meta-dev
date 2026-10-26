use std::fmt;

use crate::scope::Scope;
use affected::{CrateLoad, Packing};
use task::Failure;
use task::FullRun;
use task::Shard;
use task::Workspace;
use task::{CargoArgument, CommandLine};
use task::{Interval, Policy};

#[derive(Debug)]
pub struct ShardPlan(Vec<Shard>);

impl ShardPlan {
    pub fn full(cargo: CommandLine, policy: &Policy, trigger: Option<FullRun>) -> Result<ShardPlan, Failure> {
        policy.mutation().due(trigger).map_or_else(
            || {
                eprintln!("mutants: full run not due for this trigger");
                Ok(ShardPlan(Vec::new()))
            },
            |_| {
                let listing = cargo.argument(CargoArgument::Mutants);
                Ok(ShardPlan(Packing::shards(&CrateLoad::listed(listing, policy)?, policy, Interval::FullJob)))
            },
        )
    }

    pub fn changed_lines(workspace: &Workspace, policy: &Policy) -> Result<ShardPlan, Failure> {
        Scope::changed_shards(workspace, policy).map(ShardPlan)
    }
}

impl fmt::Display for ShardPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let spelled: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        formatter.write_str(&serde_json::to_string(&spelled).map_err(|_| fmt::Error)?)
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandLine, FullRun, Interval, Policy, ShardPlan};
    use settings::EmbeddedDefaults;
    use std::fs;
    use std::path::Path;
    use task::Program;
    const TIMEOUT: &str = "timeout-minutes: ";
    const STEP_LIMIT: &str = "        timeout-minutes: 5";
    const STEP_LIMIT_TOO_LONG: &str = "        timeout-minutes: 9";
    const SECONDS_PER_MINUTE: u64 = 60;
    const WORKFLOWS: [(&str, Interval); 2] = [
        ("../../.github/workflows/ci.yml", Interval::DiffJob),
        ("../../.github/workflows/mutants.yml", Interval::FullJob),
    ];
    const FAILING: &str = "false";
    const EMPTY: &str = "[]";
    const SHARDS: [&str; 2] = ["log/0/2", "schema+state"];
    const JSON: &str = r#"["log/0/2","schema+state"]"#;

    #[test]
    fn a_full_plan_lists_mutants_only_when_the_cadence_names_the_trigger() {
        let policy = Policy::defaults();
        let lister = || CommandLine::new(Program::Executable(FAILING.into()));
        let skipped = ShardPlan::full(lister(), &policy, None).map(|plan| plan.to_string());
        assert!(
            skipped.is_ok_and(|plan| plan == EMPTY)
                && ShardPlan::full(lister(), &policy, Some(FullRun::OnDemand)).is_err()
        );
    }

    #[test]
    fn a_plan_prints_its_shards_as_a_json_array_for_the_workflow_matrix() {
        let plan = ShardPlan(SHARDS.iter().map(|shard| shard.parse().unwrap()).collect());
        assert_eq!((plan.to_string(), ShardPlan(Vec::new()).to_string()), (JSON.to_owned(), String::from("[]")));
    }

    #[test]
    fn each_mutants_job_timeout_is_its_policy_limit_and_each_step_timeout_one_minute_less() {
        let policy = Policy::defaults();
        let fits = |text: &str, job: Interval| {
            let limit = policy.interval(job).whole_seconds() / SECONDS_PER_MINUTE;
            let found: Vec<(usize, u64)> = text
                .lines()
                .filter_map(|line| line.trim_start().strip_prefix(TIMEOUT).map(|minutes| (line, minutes)))
                .map(|(line, minutes)| (line.len() - line.trim_start().len(), minutes.trim().parse().unwrap()))
                .collect();
            let job_indent = found.iter().map(|(indent, _)| *indent).min();
            let step = |indent| u64::from(Some(indent) != job_indent);
            found.len() > 1 && found.iter().all(|(indent, minutes)| minutes + step(*indent) == limit)
        };
        let read = |workflow: &str| fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(workflow)).unwrap();
        assert!(WORKFLOWS.iter().all(|(workflow, job)| fits(&read(workflow), *job)));
        assert!(!fits(&read(WORKFLOWS[0].0).replace(STEP_LIMIT, STEP_LIMIT_TOO_LONG), WORKFLOWS[0].1));
    }
}
