use crate::facts::Facts;

pub trait FactSource {
    fn facts(&self) -> Facts;
}
