use meta_schema::Scenario;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Evidence {
    Absent,
    Measured,
    FailedRemedy,
    Scenario(Scenario),
}
