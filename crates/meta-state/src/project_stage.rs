use strum::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
#[strum(serialize_all = "lowercase")]
pub enum ProjectStage {
    Proposed,
    Accepted,
    Instrumented,
    Active,
    Delivered,
    Regressed,
}
