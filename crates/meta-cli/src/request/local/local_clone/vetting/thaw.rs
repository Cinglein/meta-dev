mod thawed;

pub use thawed::Thawed;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Thaw(Vec<Thawed>);

impl Thaw {
    pub fn new(thawed: Vec<Thawed>) -> Self {
        Self(thawed)
    }

    pub fn files(&self) -> &[Thawed] {
        &self.0
    }
}
