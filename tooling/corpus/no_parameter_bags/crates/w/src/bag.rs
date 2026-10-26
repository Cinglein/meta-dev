pub(crate) struct Bag { arm: Plies } impl Bag { pub fn new(arm: Plies) -> Bag { Bag { arm } } pub fn arm(&self) -> Plies { self.arm } pub fn again(self, arm: Plies) -> Bag { Bag { arm, ..self } } } //~ no-parameter-bags: Bag only carries
mod inner { pub(crate) struct Crate { arm: Plies } impl Crate { pub fn new(arm: Plies) -> Crate { Crate { arm } } } } //~ no-parameter-bags: Crate only carries
#[cfg(test)] mod tests { pub(crate) struct Probe { arm: Plies } }
