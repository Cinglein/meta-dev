use strum::{Display, EnumString};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Display, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum TopDivision {
    FeaturesAndMaintenance,
    PerformanceAndCost,
    ArchitectureAndPlanning,
}
