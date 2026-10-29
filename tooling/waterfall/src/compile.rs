use std::fmt;

use crate::seconds::Seconds;
use crate::section::Section;
use crate::timed_unit::TimedUnit;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Compile {
    whole: Seconds,
    frontend: Seconds,
    codegen: Seconds,
}

impl Compile {
    pub(crate) fn plus(self, unit: &TimedUnit) -> Compile {
        Compile {
            whole: self.whole + unit.section(Section::Whole),
            frontend: self.frontend + unit.section(Section::Frontend),
            codegen: self.codegen + unit.section(Section::Codegen),
        }
    }

    pub(crate) fn whole(&self) -> Seconds {
        self.whole
    }
}

impl fmt::Display for Compile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} (frontend {}, codegen {}, rest {})",
            self.whole,
            self.frontend,
            self.codegen,
            self.whole - self.frontend - self.codegen
        )
    }
}
