use super::super::kind::Kind;
use super::super::segment::Segment;

#[derive(Clone, Copy)]
pub enum Bracket {
    Open,
    Close,
}

impl Bracket {
    const OPEN: &'static str = "(";
    const CLOSE: &'static str = ")";

    pub fn found_in(c: char) -> Option<Self> {
        match c {
            '(' => Some(Self::Open),
            ')' => Some(Self::Close),
            _ => None,
        }
    }

    pub fn segment(self) -> Segment {
        Segment::default().appended(
            match self {
                Self::Open => Self::OPEN,
                Self::Close => Self::CLOSE,
            }
            .into(),
            Kind::Literal,
        )
    }
}
