mod dispatch_issuers;
mod entries;
mod liveness;
mod queue_landers;

use std::collections::BTreeMap;

use enum_map::EnumMap;
use meta_severity::Scales;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use super::bit_boundary::BitBoundary;
use super::least::Least;
use super::policy_failure::PolicyFailure;
use super::supply_chain::SupplyChain;
use super::{BaselineFile, Floor, Limit, Most, Policy, Share, SpecSettings, Surface};
use crate::branches::{Branch, BranchName};
use crate::budget::{AvailableGib, Interval, MemoryShare, Millis, MutantMillis, MutationCadence};
use crate::budgets::{Amount, BudgetDrafts, Budgets, LegacyKey, Measure};
use crate::domains::Domains;
use crate::gate_env_name::GateEnvName;
use crate::gate_status::GateStatus;
use crate::percent::Percent;
pub use dispatch_issuers::DispatchIssuers;
use entries::Entries;
pub use liveness::Liveness;
pub use queue_landers::QueueLanders;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyFile {
    vague: Vec<String>,
    wire_types: Vec<String>,
    bit_boundary: BitBoundary,
    surfaces: Vec<Surface>,
    branches: EnumMap<Branch, BranchName>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    baseline: Option<BaselineFile>,
    #[serde(default)]
    gate: Option<GateStatus>,
    #[serde(default)]
    gate_wait: Option<Millis>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    gate_env: Vec<GateEnvName>,
    limits: Entries<Limit, Most>,
    floors: Entries<Floor, Least>,
    shares: Entries<Share, Percent>,
    memory_gib: Entries<MemoryShare, AvailableGib>,
    millis: Entries<Interval, Millis>,
    mutant_millis: MutantMillis,
    mutation: MutationCadence,
    #[serde(default)]
    domains: Domains,
    #[serde(default)]
    scales: Scales,
    #[serde(default)]
    budgets: BudgetDrafts,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    spec: Option<SpecSettings>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    liveness: Option<Liveness>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    queue: Option<QueueLanders>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    dispatch: Option<DispatchIssuers>,
    supply_chain: SupplyChain,
}

impl PolicyFile {
    #[must_use]
    pub fn written(policy: &Policy) -> PolicyFile {
        PolicyFile {
            vague: policy.vague.clone(),
            wire_types: policy.wire_types.clone(),
            bit_boundary: policy.bit_boundary.clone(),
            surfaces: policy.surfaces.clone(),
            branches: policy.branches.clone(),
            baseline: None,
            gate: policy.gate.clone(),
            gate_wait: policy.gate_wait,
            gate_env: policy.gate_env.clone(),
            limits: Entries::without(&policy.limits, |limit| LegacyKey::of_limit(limit).is_some()),
            floors: Entries::without(&policy.floors, |floor| LegacyKey::of_floor(floor).is_some()),
            shares: Entries::without(&policy.shares, |share| LegacyKey::of_share(share).is_some()),
            memory_gib: Entries::without(&policy.memory_gib, |_| false),
            millis: Entries::without(&policy.millis, |interval| {
                LegacyKey::of_interval(interval).is_some()
            }),
            mutant_millis: policy.mutant_millis.clone(),
            mutation: policy.mutation.clone(),
            domains: policy.domains.clone(),
            scales: policy.scales.clone(),
            budgets: BudgetDrafts::from(policy.budgets.clone()),
            spec: policy.spec.clone(),
            liveness: policy.liveness,
            queue: policy.queue.clone(),
            dispatch: policy.dispatch.clone(),
            supply_chain: policy.supply_chain.clone(),
        }
    }

    fn legacy(&self, key: LegacyKey) -> Option<Amount> {
        let millis =
            |interval: Interval| self.millis.entry(interval).copied().map(Amount::Duration);
        match key {
            LegacyKey::Total => millis(Interval::Total),
            LegacyKey::PerTest => millis(Interval::PerTest),
            LegacyKey::FullJob => millis(Interval::FullJob),
            LegacyKey::DiffJob => millis(Interval::DiffJob),
            LegacyKey::PriceTolerance => self
                .floors
                .entry(Floor::PriceTolerance)
                .map(|_| Amount::Share(Percent::ALL)),
            LegacyKey::TestLinePercent => self
                .shares
                .entry(Share::TestLinePercent)
                .copied()
                .map(Amount::Share),
            LegacyKey::SccItems => self
                .limits
                .entry(Limit::SccItems)
                .copied()
                .map(Amount::Count),
            LegacyKey::Baseline => self.baseline.clone().map(Amount::Baseline),
        }
    }

    fn placed(&self, key: LegacyKey) -> Result<(), PolicyFailure> {
        match (self.legacy(key), self.budgets.measures(&key.measure())) {
            (Some(_), true) => Err(PolicyFailure::Twice(key)),
            (None, false) => Err(PolicyFailure::Unset(key)),
            (Some(_), false) | (None, true) => Ok(()),
        }
    }

    fn budgeted(&self) -> Result<Budgets, PolicyFailure> {
        LegacyKey::iter().try_for_each(|key| self.placed(key))?;
        let tolerance = self
            .floors
            .entry(Floor::PriceTolerance)
            .map(|least| Percent::from(usize::from(*least)));
        Ok(LegacyKey::iter()
            .filter_map(|key| self.legacy(key).map(|amount| (key, amount)))
            .try_fold(self.budgets.clone(), |budgets, (key, amount)| {
                budgets.adding(
                    key,
                    amount,
                    tolerance.filter(|_| key == LegacyKey::PriceTolerance),
                )
            })
            .map(BTreeMap::from)
            .and_then(Budgets::try_from)?)
    }

    pub fn resolved(self) -> Result<Policy, PolicyFailure> {
        let budgets = self.budgeted()?;
        let deprecated = LegacyKey::iter()
            .filter(|key| self.legacy(*key).is_some())
            .collect();
        budgets.measurable(&self.domains)?;
        budgets.leveled(&self.scales)?;
        let amount =
            |key: Option<LegacyKey>| key.and_then(|key| budgets.blocking_line(&key.measure()));
        let baseline = budgets
            .whole(&Measure::LintFindings)
            .and_then(|budget| budget.allowance().whole().cloned());
        let tolerance = budgets
            .whole(&Measure::MutantPrice)
            .map(|budget| budget.tolerance().unwrap_or(Percent::NONE));
        Ok(Policy {
            vague: self.vague,
            wire_types: self.wire_types,
            bit_boundary: self.bit_boundary,
            surfaces: self.surfaces,
            branches: self.branches,
            baseline: baseline
                .as_ref()
                .and_then(Amount::baseline)
                .cloned()
                .ok_or(PolicyFailure::Missing(Self::BASELINE))?,
            price_line: budgets
                .blocking_line(&Measure::MutantPrice)
                .as_ref()
                .and_then(Amount::share)
                .ok_or(PolicyFailure::Missing(Self::PRICE))?,
            gate: self.gate,
            gate_wait: self.gate_wait,
            gate_env: self.gate_env,
            limits: self
                .limits
                .completed(|limit| {
                    amount(LegacyKey::of_limit(limit))
                        .as_ref()
                        .and_then(Amount::count)
                })
                .map_err(|key| PolicyFailure::MissingKey(Self::LIMITS, key))?,
            floors: self
                .floors
                .completed(|floor| {
                    LegacyKey::of_floor(floor)
                        .and(tolerance)
                        .map(|percent| Least::from(usize::from(percent)))
                })
                .map_err(|key| PolicyFailure::MissingKey(Self::FLOORS, key))?,
            shares: self
                .shares
                .completed(|share| {
                    amount(LegacyKey::of_share(share))
                        .as_ref()
                        .and_then(Amount::share)
                })
                .map_err(|key| PolicyFailure::MissingKey(Self::SHARES, key))?,
            memory_gib: self
                .memory_gib
                .completed(|_| None)
                .map_err(|key| PolicyFailure::MissingKey(Self::MEMORY_GIB, key))?,
            millis: self
                .millis
                .completed(|interval| {
                    amount(LegacyKey::of_interval(interval))
                        .as_ref()
                        .and_then(Amount::duration)
                })
                .map_err(|key| PolicyFailure::MissingKey(Self::MILLIS, key))?,
            mutant_millis: self.mutant_millis,
            mutation: self.mutation,
            domains: self.domains,
            scales: self.scales,
            budgets,
            deprecated,
            spec: self.spec,
            liveness: self.liveness,
            queue: self.queue,
            dispatch: self.dispatch,
            supply_chain: self.supply_chain,
        })
    }

    const BASELINE: &str = "baseline";
    const PRICE: &str = "budgets.mutant-price";
    const LIMITS: &str = "limits";
    const FLOORS: &str = "floors";
    const SHARES: &str = "shares";
    const MILLIS: &str = "millis";
    const MEMORY_GIB: &str = "memory_gib";
}
