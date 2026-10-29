use meta_schema::Scenario;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Evidence {
    Absent,
    Measured,
    FailedRemedy,
    Scenario(Scenario),
}

impl Evidence {
    #[must_use]
    pub fn supports_raise(&self) -> bool {
        !matches!(self, Self::Absent)
    }
}
