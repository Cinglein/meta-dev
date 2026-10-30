use std::fmt;

use crate::write_kind::WriteKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FieldKind {
    Flag,
    Mode,
    Slot,
    Plain,
}

impl FieldKind {
    #[must_use]
    pub fn hiding_writes(self) -> &'static [WriteKind] {
        match self {
            FieldKind::Flag | FieldKind::Mode => &[WriteKind::Assigned],
            FieldKind::Slot => &[WriteKind::Assigned, WriteKind::Filled],
            FieldKind::Plain => &[],
        }
    }
}

impl fmt::Display for FieldKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            FieldKind::Flag => "bool",
            FieldKind::Mode => "local enum",
            FieldKind::Slot => "Option",
            FieldKind::Plain => "plain",
        })
    }
}
