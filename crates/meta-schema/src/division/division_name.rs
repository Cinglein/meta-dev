use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DivisionName(String);

impl DivisionName {
    pub fn named(name: &str) -> Option<DivisionName> {
        let spelled = |letter: char| letter.is_ascii_lowercase() || letter.is_ascii_digit() || letter == '-';
        (!name.is_empty() && name.chars().all(spelled)).then(|| DivisionName(name.to_owned()))
    }
}

impl fmt::Display for DivisionName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
