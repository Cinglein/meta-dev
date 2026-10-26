use std::fmt;
use std::num::NonZeroU64;

use task::Millis;
use task::Policy;

#[derive(Debug)]
pub struct Overrun {
    name: String,
    measured: Millis,
    price: Millis,
}

impl Overrun {
    #[must_use]
    pub fn new(name: String, measured: Millis, price: Millis) -> Overrun {
        Overrun { name, measured, price }
    }

    #[must_use]
    pub fn rounded_up_to(self, step: NonZeroU64) -> Overrun {
        Overrun { measured: self.measured.rounded_up_to(step), ..self }
    }

    #[must_use]
    pub fn is_above_price(&self) -> bool {
        self.measured > self.price
    }

    #[must_use]
    pub fn is_beyond_tolerance(&self, policy: &Policy) -> bool {
        self.measured > policy.tolerated(self.price)
    }
}

impl fmt::Display for Overrun {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} above price {}", self.name, self.measured, self.price)
    }
}
