use std::cmp::Ordering;
use std::fmt;

use crate::compile::Compile;
use task::CrateName;

#[derive(Debug)]
pub(crate) struct CrateTime<'unit> {
    name: &'unit CrateName,
    compile: Compile,
}

impl<'unit> CrateTime<'unit> {
    pub(crate) fn new(name: &'unit CrateName, compile: Compile) -> CrateTime<'unit> {
        CrateTime { name, compile }
    }

    pub(crate) fn slower(&self, other: &CrateTime<'_>) -> Ordering {
        other.compile.whole().ordered(self.compile.whole())
    }
}

impl fmt::Display for CrateTime<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.name, self.compile)
    }
}
