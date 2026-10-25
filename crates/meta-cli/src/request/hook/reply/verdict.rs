use super::blocking::Blocking;
use super::denial::Denial;

#[derive(Debug, Default)]
pub enum Verdict {
    #[default]
    Allow,
    Deny(Denial),
    Block(Blocking),
}

impl Verdict {
    pub fn denial(&self) -> Option<&Denial> {
        match self {
            Self::Deny(reason) => Some(reason),
            Self::Allow | Self::Block(_) => None,
        }
    }

    pub fn blocking(&self) -> Option<&Blocking> {
        match self {
            Self::Block(reason) => Some(reason),
            Self::Allow | Self::Deny(_) => None,
        }
    }

    pub fn allowing(&self) -> Option<&Self> {
        match self {
            Self::Allow => Some(self),
            Self::Deny(_) | Self::Block(_) => None,
        }
    }
}
