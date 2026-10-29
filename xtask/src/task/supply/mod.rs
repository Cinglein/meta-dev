mod deny_config;
mod runner;
mod supply_argument;

use task::{Branch, CargoTool, Policy, Program, Selection, Workspace};

use crate::task::failure::Failure;
use crate::task::revision::Revision;
use deny_config::DenyConfig;
pub use runner::Runner;
use supply_argument::SupplyArgument;

#[derive(Debug)]
pub struct Supply;

impl Supply {
    const ADVISORIES: &str = "deny advisories";
    const EVERY_CHECK: [SupplyArgument; 4] =
        [SupplyArgument::Advisories, SupplyArgument::Bans, SupplyArgument::Licenses, SupplyArgument::Sources];
    const OFFLINE_CHECKS: [SupplyArgument; 3] =
        [SupplyArgument::Bans, SupplyArgument::Licenses, SupplyArgument::Sources];
    const DENY_TOOL: &str = "cargo-deny 0.20.2";
    const SEMVER_TOOL: &str = "cargo-semver-checks 0.51.0";

    pub fn deny(workspace: &Workspace, policy: &Policy) -> Result<(), Failure> {
        let config = DenyConfig::written(workspace, policy)?;
        let deny = || {
            workspace
                .line(Program::Cargo)
                .argument(SupplyArgument::Deny)
                .argument(SupplyArgument::Config)
                .argument(&config)
        };
        let fetched = deny().argument(SupplyArgument::Fetch).argument(SupplyArgument::Database).run();
        let checks = match fetched {
            Ok(()) => &Self::EVERY_CHECK[..],
            Err(failure) => Runner::ambient()
                .unreached(Failure::from(failure), Self::ADVISORIES)
                .map(|()| &Self::OFFLINE_CHECKS[..])?,
        };
        deny()
            .argument(SupplyArgument::Check)
            .argument(SupplyArgument::HideGraph)
            .arguments(checks)
            .run()
            .map_err(|failure| Failure::Supply { tool: Self::DENY_TOOL, failure: Box::new(failure) })
    }

    pub fn semver(workspace: &Workspace, selection: &Selection, policy: &Policy) -> Result<(), Failure> {
        let checked = workspace.packages().published_in(selection, policy.supply_chain().semver());
        checked.reaching().map_or_else(
            || {
                println!("semver: no published or depended-on crate affected");
                Ok(())
            },
            |checked| {
                let base = Revision::merge_base(workspace.root(), policy.branch(Branch::Base))?;
                workspace
                    .line(Program::Cargo)
                    .argument(SupplyArgument::SemverChecks)
                    .arguments(checked.arguments::<CargoTool>())
                    .argument(SupplyArgument::BaselineRevision)
                    .argument(base.commit())
                    .run()
                    .map_err(|failure| Failure::Supply { tool: Self::SEMVER_TOOL, failure: Box::new(failure) })
            },
        )
    }
}
