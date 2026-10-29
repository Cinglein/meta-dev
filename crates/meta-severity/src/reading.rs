use std::fmt;

use meta_schema::{Division, Rating};

use crate::rating_failure::RatingFailure;
use crate::scale::Scale;
use crate::scales::Scales;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading {
    Clear,
    Rating(Rating),
}

impl Reading {
    #[must_use]
    pub fn rolled_up(&self, scales: &Scales, from: &Division, onto: &Division) -> Reading {
        match self {
            Self::Clear => Self::Clear,
            Self::Rating(rating) => {
                Reading::from(Rating::from(rating.level().and_then(|level| scales.rolled_up(from, level, onto))))
            }
        }
    }

    pub fn index_on(&self, scale: &Scale) -> Result<Option<usize>, RatingFailure> {
        match self {
            Self::Clear => Ok(Some(scale.levels().len())),
            Self::Rating(rating) => rating.level().map(|level| scale.known_index(level)).transpose(),
        }
    }

    #[must_use]
    pub fn known(&self) -> Option<&Reading> {
        match self {
            Self::Clear => Some(self),
            Self::Rating(rating) => rating.level().map(|_| self),
        }
    }
}

impl From<Rating> for Reading {
    fn from(rating: Rating) -> Reading {
        Reading::Rating(rating)
    }
}

impl fmt::Display for Reading {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Clear => formatter.write_str("clear"),
            Self::Rating(rating) => rating.fmt(formatter),
        }
    }
}
