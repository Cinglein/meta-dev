use crate::command::output::Listed;

#[derive(Debug, Default)]
pub enum Verdict {
    #[default]
    Allow,
    Deny(Listed),
    Block(Listed),
}

impl Verdict {
    pub fn denial(&self) -> Option<&Listed> {
        match self {
            Self::Deny(reason) => Some(reason),
            Self::Allow | Self::Block(_) => None,
        }
    }

    pub fn blocking(&self) -> Option<&Listed> {
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
