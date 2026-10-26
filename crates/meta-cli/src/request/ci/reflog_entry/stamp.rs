use super::UnixSeconds;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stamp(String);

impl Stamp {
    const OPEN: char = '{';
    const CLOSE: char = '}';

    pub fn new(identity: &str, at: &str) -> Self {
        Self(format!("{identity} {at}"))
    }

    pub fn from_selector(identity: &str, selector: &str) -> Self {
        let at = selector.split_once(Self::OPEN).map_or(selector, |(_, at)| at.trim_end_matches(Self::CLOSE));
        Self::new(identity, at)
    }

    pub fn at(&self) -> Option<UnixSeconds> {
        self.0.rsplit_once(' ').and_then(|(_, at)| at.parse().ok())
    }
}
