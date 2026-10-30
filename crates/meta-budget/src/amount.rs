use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::baseline_file::BaselineFile;
use crate::millis::Millis;
use crate::most::Most;
use crate::percent::Percent;
use crate::unit::Unit;
use crate::vocabulary_failure::VocabularyFailure;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Amount {
    Count(Most),
    Duration(Millis),
    Share(Percent),
    Baseline(BaselineFile),
}

impl Amount {
    #[must_use]
    pub const fn unit(&self) -> Unit {
        match self {
            Amount::Count(_) => Unit::Count,
            Amount::Duration(_) => Unit::Duration,
            Amount::Share(_) => Unit::Share,
            Amount::Baseline(_) => Unit::Baseline,
        }
    }

    #[must_use]
    pub fn part(&self, percent: Percent) -> Option<Amount> {
        match self {
            Amount::Count(most) => percent.scaled(most.count()).map(Amount::Count),
            Amount::Duration(millis) => millis.scaled(percent).map(Amount::Duration),
            Amount::Share(share) => percent.scaled(*share).map(Amount::Share),
            Amount::Baseline(_) => None,
        }
    }

    #[must_use]
    pub fn duration(&self) -> Option<Millis> {
        match self {
            Amount::Duration(millis) => Some(*millis),
            Amount::Count(_) | Amount::Share(_) | Amount::Baseline(_) => None,
        }
    }

    #[must_use]
    pub fn count(&self) -> Option<Most> {
        match self {
            Amount::Count(most) => Some(*most),
            Amount::Duration(_) | Amount::Share(_) | Amount::Baseline(_) => None,
        }
    }

    #[must_use]
    pub fn share(&self) -> Option<Percent> {
        match self {
            Amount::Share(percent) => Some(*percent),
            Amount::Count(_) | Amount::Duration(_) | Amount::Baseline(_) => None,
        }
    }

    #[must_use]
    pub fn baseline(&self) -> Option<&BaselineFile> {
        match self {
            Amount::Baseline(file) => Some(file),
            Amount::Count(_) | Amount::Duration(_) | Amount::Share(_) => None,
        }
    }
}

impl FromStr for Amount {
    type Err = VocabularyFailure;

    fn from_str(text: &str) -> Result<Amount, VocabularyFailure> {
        let unreadable = |_| VocabularyFailure::UnreadableAmount;
        match (text.strip_suffix("ms"), text.strip_suffix('%')) {
            (Some(millis), _) => millis.parse().map(|millis| Amount::Duration(Millis::new(millis))).map_err(unreadable),
            (_, Some(share)) => share.parse().map(|share| Amount::Share(Percent::new(share))).map_err(unreadable),
            (None, None) => text.parse::<usize>().map(|count| Amount::Count(Most::from(count))).map_err(unreadable),
        }
    }
}

impl PartialOrd for Amount {
    fn partial_cmp(&self, other: &Amount) -> Option<Ordering> {
        match (self, other) {
            (Amount::Count(spent), Amount::Count(line)) => spent.partial_cmp(line),
            (Amount::Duration(spent), Amount::Duration(line)) => spent.partial_cmp(line),
            (Amount::Share(spent), Amount::Share(line)) => spent.partial_cmp(line),
            _ => (self == other).then_some(Ordering::Equal),
        }
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Amount::Count(most) => write!(formatter, "{most}"),
            Amount::Duration(millis) => write!(formatter, "{millis}ms"),
            Amount::Share(percent) => write!(formatter, "{percent}%"),
            Amount::Baseline(file) => write!(formatter, "{}", file.path().display()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::Amount;
    use crate::{BaselineFile, Most, Percent};

    const FEW: usize = 3;
    const MANY: usize = 5;
    const FILE: &str = "baseline.toml";
    const OTHER_FILE: &str = "other.toml";

    #[test]
    fn amounts_order_within_a_unit_and_only_an_equal_baseline_compares_across() {
        let [few, many] = [FEW, MANY].map(|count| Amount::Count(Most::from(count)));
        let [small, large] = [FEW, MANY].map(|share| Amount::Share(Percent::from(share)));
        let [file, other] = [FILE, OTHER_FILE].map(|path| Amount::Baseline(BaselineFile::from(path)));
        assert!(many > few && large > small && few.partial_cmp(&small).is_none());
        assert_eq!([file.partial_cmp(&file.clone()), file.partial_cmp(&other)], [Some(Ordering::Equal), None]);
    }
}
