use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Lower,
    Higher,
    TargetZero,
}

impl fmt::Display for Direction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Direction::Lower => "lower",
            Direction::Higher => "higher",
            Direction::TargetZero => "to zero",
        })
    }
}
