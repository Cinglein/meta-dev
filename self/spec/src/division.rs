use strum::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
#[strum(serialize_all = "kebab-case")]
pub enum Division {
    FeaturesAndMaintenance,
    PerformanceAndCost,
    ArchitectureAndPlanning,
}
