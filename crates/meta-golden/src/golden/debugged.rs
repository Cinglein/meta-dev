use std::fmt;

use super::Golden;
use crate::golden_failure::GoldenFailure;

struct Debugged<'value, Value: ?Sized>(&'value Value);

impl<Value: fmt::Debug + ?Sized> fmt::Display for Debugged<'_, Value> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{:#?}", self.0)
    }
}

impl Golden {
    pub fn debugged(&self, value: &(impl fmt::Debug + ?Sized)) -> Result<(), GoldenFailure> {
        self.shown(&Debugged(value))
    }

    pub fn shown(&self, value: &impl fmt::Display) -> Result<(), GoldenFailure> {
        self.checked(&value.to_string())
    }
}
