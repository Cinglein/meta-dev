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

#[cfg(test)]
mod tests {
    use super::Wording;

    const ONE_LINE: &str = "One line.";
    const TWO_LINES: &str = "One line.\n- `a-rule` (performance-and-cost, rung 1, no enforcer): Posing.";

    #[test]
    #[should_panic(expected = "one line")]
    fn wording_with_a_newline_is_refused() {
        assert_eq!(Wording::new(ONE_LINE).to_string(), ONE_LINE);
        let _ = Wording::new(TWO_LINES);
    }
}
