use std::fmt;

use super::module_path::ModulePath;
use super::reexport_limit::ReexportLimit;
use crate::count::Count;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NestingMessage {
    Misplaced(ModulePath),
    DeepUse(ModulePath),
    Crowded(Count),
}

impl fmt::Display for NestingMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NestingMessage::Misplaced(home) => write!(formatter, "used only from {home}; move it inside that module"),
            NestingMessage::DeepUse(target) => {
                write!(formatter, "reaches into {target}; import the parent's re-export instead")
            }
            NestingMessage::Crowded(count) => {
                write!(formatter, "re-exports {count} items, at most {} allowed; split the module", ReexportLimit::MAX)
            }
        }
    }
}
