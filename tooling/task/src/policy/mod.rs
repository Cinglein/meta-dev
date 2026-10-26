mod available_gib;
mod bit_boundary;
mod compiler_tasks;
mod floor;
mod interval;
mod least;
mod limit;
mod memory_share;
mod millis;
mod most;
mod percent;
mod share;
mod vague;
mod vocabulary;
mod wire_type;

pub use floor::Floor;
pub use interval::Interval;
pub use limit::Limit;
pub use millis::Millis;
pub use most::Most;
pub use share::Share;
pub use wire_type::WireType;

use available_gib::AvailableGib;
use bit_boundary::BitBoundary;
use compiler_tasks::CompilerTasks;
use enum_map::EnumMap;
use least::Least;
use memory_share::MemoryShare;
use percent::Percent;
use serde::{Deserialize, Serialize};
use settings::EmbeddedDefaults;
use vague::Vague;
use vocabulary::Vocabulary;

use crate::name::Name;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    vague: Vec<String>,
    wire_types: Vec<String>,
    bit_boundary: BitBoundary,
    limits: EnumMap<Limit, Most>,
    floors: EnumMap<Floor, Least>,
    shares: EnumMap<Share, Percent>,
    memory_gib: EnumMap<MemoryShare, AvailableGib>,
    millis: EnumMap<Interval, Millis>,
}

impl EmbeddedDefaults for Policy {
    const DEFAULTS: &str = include_str!("../../../policy.toml");
}

impl Policy {
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
    pub fn compiler_tasks(&self) -> CompilerTasks {
        CompilerTasks::within(
            self.memory_gib[MemoryShare::Granted].minus(self.memory_gib[MemoryShare::ForTheTests]),
            self.memory_gib[MemoryShare::PerCompilerTask],
        )
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
    const CHECK: &str = "check";
    const LONG_ENOUGH: &str = "report";

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
}
