use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(from = "bool")]
pub enum Doctests {
    Run,
    Skipped,
}

impl From<bool> for Doctests {
    fn from(doctest: bool) -> Doctests {
        if doctest {
            Doctests::Run
        } else {
            Doctests::Skipped
        }
    }
}
