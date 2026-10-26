mod identity;

use std::str::FromStr;

use super::UnixSeconds;
use identity::Identity;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stamp {
    identity: Identity,
    moment: UnixSeconds,
}

impl Stamp {
    const SELECTOR_OPEN: char = '{';
    const SELECTOR_CLOSE: char = '}';

    pub fn not_before(&self, start: UnixSeconds) -> bool {
        self.moment >= start
    }
}

impl FromStr for Stamp {
    type Err = std::num::ParseIntError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (identity, moment) = text.rsplit_once(' ').unwrap_or_default();
        let moment = moment.split_once(Self::SELECTOR_OPEN).map_or(moment, |(_, inner)| inner);
        let Ok(identity) = identity.parse();
        Ok(Self { identity, moment: moment.trim_end_matches(Self::SELECTOR_CLOSE).parse()? })
    }
}
