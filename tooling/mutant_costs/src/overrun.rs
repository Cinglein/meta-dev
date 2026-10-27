use std::fmt;

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

#[cfg(test)]
mod tests {
    use super::Overrun;
    use settings::EmbeddedDefaults;
    use task::Millis;
    use task::Policy;
    const NAME: &str = "crate";
    const SPELLED: &str = "crate 5 above price 3";
    const PRICE: Millis = Millis::new(40);

    #[test]
    fn an_overrun_spells_its_name_measurement_and_price() {
        let overrun = Overrun::new(NAME.to_owned(), Millis::new(5), Millis::new(3));
        assert_eq!(overrun.to_string(), SPELLED);
    }

    #[test]
    fn a_measurement_equal_to_the_price_is_not_above_it() {
        let overrun = Overrun::new(NAME.to_owned(), PRICE, PRICE);
        assert!(!overrun.is_above_price());
    }

    #[test]
    fn a_measurement_equal_to_the_tolerated_line_is_not_beyond_tolerance() {
        let policy = Policy::defaults();
        let overrun = Overrun::new(NAME.to_owned(), policy.tolerated(PRICE), PRICE);
        assert!(!overrun.is_beyond_tolerance(&policy));
    }
}
