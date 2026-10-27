use strum::{Display, EnumIter, EnumString};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Display, EnumString, EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum TopDivision {
    FeaturesAndMaintenance,
    PerformanceAndCost,
    ArchitectureAndPlanning,
}
