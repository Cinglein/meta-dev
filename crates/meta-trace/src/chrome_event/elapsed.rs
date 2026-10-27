use std::fmt;
use std::ops::Add;

use serde::{Deserialize, Serialize};

use crate::micros::Micros;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Elapsed(Micros);

impl Elapsed {
    pub fn between(start: Micros, end: Micros) -> Self {
        Self(end.since(start))
    }
}

impl Add<Elapsed> for Micros {
    type Output = Micros;

    fn add(self, elapsed: Elapsed) -> Micros {
        self + elapsed.0
    }
}

impl fmt::Display for Elapsed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
