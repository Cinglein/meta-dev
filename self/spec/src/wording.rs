use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wording(&'static str);

impl Wording {
    #[must_use]
    pub const fn new(text: &'static str) -> Wording {
        Wording(text)
    }
}

impl fmt::Display for Wording {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}
