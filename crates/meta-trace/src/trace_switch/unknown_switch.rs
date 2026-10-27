use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnknownSwitch;

impl fmt::Display for UnknownSwitch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("META_TRACE must be on or off")
    }
}

impl std::error::Error for UnknownSwitch {}
