use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Conductance(f64);

impl Conductance {
    pub fn of_ratio(crossing: f64, volume: f64) -> Conductance {
        Conductance(if volume > 0.0 { crossing / volume } else { 0.0 })
    }

    pub fn ratio(self) -> f64 {
        self.0
    }

    pub fn total_cmp(self, other: Conductance) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl fmt::Display for Conductance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.3}", self.0)
    }
}
