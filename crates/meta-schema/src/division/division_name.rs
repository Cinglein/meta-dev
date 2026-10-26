use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DivisionName(String);

impl DivisionName {
    pub fn named(name: &str) -> Option<DivisionName> {
        (!name.is_empty()).then(|| DivisionName(name.to_owned()))
    }
}

impl fmt::Display for DivisionName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
