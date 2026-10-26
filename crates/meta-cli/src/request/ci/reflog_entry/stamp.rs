use std::str::FromStr;

use super::identity::Identity;
use super::{StartMoment, UnixSeconds};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stamp {
    identity: Identity,
    moment: UnixSeconds,
}

impl Stamp {
    const SELECTOR_OPEN: char = '{';
    const SELECTOR_CLOSE: char = '}';

    pub fn by_after(&self, identity: &Identity, start: &StartMoment, born_after: impl FnOnce() -> bool) -> bool {
        self.identity == *identity
            && match start.against_second(self.moment) {
                std::cmp::Ordering::Greater => true,
                std::cmp::Ordering::Equal => born_after(),
                std::cmp::Ordering::Less => false,
            }
    }
}

impl FromStr for Stamp {
    type Err = std::num::ParseIntError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (identity, selector) = text.rsplit_once(' ').unwrap_or_default();
        let moment = selector.split_once(Self::SELECTOR_OPEN).map_or(selector, |(_, inner)| inner);
        let Ok(identity) = identity.parse();
        Ok(Self { identity, moment: moment.trim_end_matches(Self::SELECTOR_CLOSE).parse()? })
    }
}
