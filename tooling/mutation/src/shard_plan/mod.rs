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
                Ok(ShardPlan(Packing::shards(
                    &CrateLoad::listed(listing, policy)?,
                    policy.interval(Interval::PerShard),
                )))
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
    use super::{CommandLine, FullRun, Policy, ShardPlan};
    use settings::EmbeddedDefaults;
    use task::Program;
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
}
