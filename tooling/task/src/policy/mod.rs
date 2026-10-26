mod baseline_file;
mod bit_boundary;
mod budgets;
mod domains;
mod floor;
mod least;
mod legacy_key;
mod limit;
mod most;
mod percent;
mod share;
mod spec_settings;
mod surface;
mod vague;
mod vocabulary;
mod wire_type;

use crate::gate_env_name::GateEnvName;
use crate::gate_status::GateStatus;
pub use baseline_file::BaselineFile;
pub use floor::Floor;
pub use limit::Limit;

pub use most::Most;
pub use share::Share;
pub use spec_settings::SpecSettings;
pub use surface::Surface;
pub use wire_type::WireType;

use std::fs;
use std::num::NonZeroU64;
use std::path::Path;

use bit_boundary::BitBoundary;
use budgets::Budgets;
use domains::Domains;
use enum_map::EnumMap;
use least::Least;
use percent::Percent;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(any(test, feature = "test-policy"))]
use settings::EmbeddedDefaults;
use vague::Vague;
use vocabulary::Vocabulary;

use crate::branches::{Branch, BranchName};
use crate::budget::{
    AvailableGib, CompilerTasks, Interval, MemoryShare, Millis, MutantMillis, MutationCadence,
};
use crate::crate_name::CrateName;
use crate::failure::Failure;
use crate::name::Name;
use crate::site::Site;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self", deny_unknown_fields)]
pub struct Policy {
    vague: Vec<String>,
    wire_types: Vec<String>,
    bit_boundary: BitBoundary,
    surfaces: Vec<Surface>,
    branches: EnumMap<Branch, BranchName>,
    baseline: BaselineFile,
    #[serde(default)]
    gate: Option<GateStatus>,
    #[serde(default)]
    gate_wait: Option<Millis>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    gate_env: Vec<GateEnvName>,
    limits: EnumMap<Limit, Most>,
    floors: EnumMap<Floor, Least>,
    shares: EnumMap<Share, Percent>,
    memory_gib: EnumMap<MemoryShare, AvailableGib>,
    millis: EnumMap<Interval, Millis>,
    mutant_millis: MutantMillis,
    mutation: MutationCadence,
    #[serde(default)]
    domains: Domains,
    #[serde(default)]
    budgets: Budgets,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    spec: Option<SpecSettings>,
}

#[cfg(any(test, feature = "test-policy"))]
impl EmbeddedDefaults for Policy {
    const DEFAULTS: &str = include_str!("../../../policy.toml");
}

impl<'de> Deserialize<'de> for Policy {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Policy, D::Error> {
        let policy = Policy::deserialize(deserializer)?;
        policy
            .budgets
            .measurable(&policy.domains)
            .map_err(D::Error::custom)?;
        Ok(policy)
    }
}

impl Serialize for Policy {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Policy::serialize(self, serializer)
    }
}

impl Policy {
    const PATH: &str = "tooling/policy.toml";
    const GATE: &str = "gate in tooling/policy.toml";
    const GATE_WAIT: &str = "gate_wait in tooling/policy.toml";
    const PRICE_STEP: NonZeroU64 = NonZeroU64::new(100).unwrap();

    pub fn read(root: &Path) -> Result<Policy, Failure> {
        let path = root.join(Self::PATH);
        let text = fs::read_to_string(&path).map_err(|error| Failure::Io {
            site: Site::File(path.clone()),
            error,
        })?;
        toml::from_str(&text).map_err(|error| Failure::Policy {
            site: Site::File(path),
            error,
        })
    }

    #[must_use]
    pub fn limit(&self, limit: Limit) -> Most {
        self.limits[limit]
    }

    #[must_use]
    pub fn floor(&self, floor: Floor) -> Least {
        self.floors[floor]
    }

    #[must_use]
    pub fn share(&self, share: Share) -> Percent {
        self.shares[share]
    }

    #[must_use]
    pub fn interval(&self, interval: Interval) -> Millis {
        self.millis[interval]
    }

    #[must_use]
    pub fn branch(&self, branch: Branch) -> &BranchName {
        &self.branches[branch]
    }

    #[must_use]
    pub fn memory(&self, share: MemoryShare) -> AvailableGib {
        self.memory_gib[share]
    }

    #[must_use]
    pub fn mutant_millis(&self) -> &MutantMillis {
        &self.mutant_millis
    }

    #[must_use]
    pub fn mutant_price(&self, name: &CrateName, job: Interval) -> Millis {
        let full = self.mutant_millis.for_crate(name);
        match job {
            Interval::DiffJob => {
                self.floors[Floor::DiffFactor].times_rounded_up(full, Self::PRICE_STEP)
            }
            _ => full,
        }
    }

    #[must_use]
    pub fn tolerated(&self, price: Millis) -> Millis {
        self.floors[Floor::PriceTolerance].percent_above(price)
    }

    #[must_use]
    pub fn mutation(&self) -> &MutationCadence {
        &self.mutation
    }

    #[must_use]
    pub fn compiler_tasks(&self) -> CompilerTasks {
        CompilerTasks::within(
            self.memory_gib[MemoryShare::Granted].minus(self.memory_gib[MemoryShare::ForTheTests]),
            self.memory_gib[MemoryShare::PerCompilerTask],
        )
    }

    #[must_use]
    pub fn budgets(&self) -> &Budgets {
        &self.budgets
    }

    #[must_use]
    pub fn spec(&self) -> Option<&SpecSettings> {
        self.spec.as_ref()
    }

    #[must_use]
    pub fn surfaces(&self) -> &[Surface] {
        &self.surfaces
    }

    #[must_use]
    pub fn baseline(&self) -> &BaselineFile {
        &self.baseline
    }

    pub fn gate(&self) -> Result<&GateStatus, Failure> {
        self.gate.as_ref().ok_or(Failure::Unset(Self::GATE))
    }

    pub fn gate_wait(&self) -> Result<Millis, Failure> {
        self.gate_wait.ok_or(Failure::Unset(Self::GATE_WAIT))
    }

    #[must_use]
    pub fn gate_env(&self) -> &[GateEnvName] {
        &self.gate_env
    }

    #[must_use]
    pub fn bit_boundary(&self) -> &BitBoundary {
        &self.bit_boundary
    }

    #[must_use]
    pub fn is_vague(&self, name: &Name) -> bool {
        !self.floor(Floor::FnName).allows(name.length()) || self.lists::<Vague>(name)
    }

    #[must_use]
    pub fn lists<V: Vocabulary>(&self, word: &impl PartialEq<String>) -> bool {
        V::entries(self).iter().any(|entry| word == entry)
    }
}

#[cfg(test)]
mod tests {
    use super::{Limit, Name, Policy};
    use settings::EmbeddedDefaults;
    use std::path::Path;
    const CHECK: &str = "check";
    const LONG_ENOUGH: &str = "report";
    const TEST_TIME: &str = "test-time";
    const BUDGET: &str = "\n[budgets.test-time]\ndivision = \"performance-and-cost\"\nmeasure = \"test_wall_time\"\nallowance = { duration = 4000 }\nwindow = \"per_change\"\nsource = \"ci\"\n";

    #[test]
    fn the_checked_in_policy_round_trips_and_knows_a_vague_name_from_a_said_one() {
        let written = toml::to_string_pretty(&Policy::defaults()).unwrap();
        let read: Policy = toml::from_str(&written).unwrap();
        assert_eq!(read, Policy::defaults());
        assert!(read.is_vague(&Name::from(CHECK)) && !read.is_vague(&Name::from(LONG_ENOUGH)));
        assert!(
            read.limit(Limit::Parameters)
                .allows(read.limit(Limit::Parameters).count())
        );
    }

    #[test]
    fn the_policy_is_read_from_the_tooling_directory_of_a_workspace_root() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        assert_eq!(Policy::read(&root).unwrap(), Policy::defaults());
        assert!(Policy::read(&root.join("tooling")).is_err());
    }

    #[test]
    fn a_policy_without_budgets_has_none_and_one_with_budgets_reads_them() {
        let budgeted: Policy = toml::from_str(&format!("{}{BUDGET}", Policy::DEFAULTS)).unwrap();
        assert!(
            Policy::defaults()
                .budgets()
                .budget(&TEST_TIME.parse().unwrap())
                .is_none()
        );
        assert!(
            budgeted
                .budgets()
                .budget(&TEST_TIME.parse().unwrap())
                .is_some()
        );
    }
}
