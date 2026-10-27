use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootRef(String);

impl RootRef {
    const PEELED: &'static str = "^{commit}";
    const ALPHABET: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ_-";

    pub fn peeled(&self) -> String {
        [self.0.as_str(), Self::PEELED].concat()
    }
}

impl FromStr for RootRef {
    type Err = ();

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        name.chars().all(|letter| Self::ALPHABET.contains(letter)).then(|| Self(name.to_owned())).ok_or(())
    }
}
