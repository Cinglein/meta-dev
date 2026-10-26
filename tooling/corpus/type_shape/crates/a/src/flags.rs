struct Flags { ready: bool, pair: (Plies, Plies) } //~ type-shape: bool field //~ type-shape: tuple in a signature
impl Flags { fn split(&self) -> (Plies, Plies) { self.pair } } //~ type-shape: tuple in a signature
impl Flags { fn load(&self) -> Result<Plies, String> { Ok(0) } fn unit(&self) -> () {} } //~ type-shape: Result with a String
impl Iterator for Flags { type Item = Plies; fn size_hint(&self) -> (usize, Option<usize>) { (0, None) } }
mod inner { struct Inner { ready: bool } } //~ type-shape: bool field
#[cfg(test)] mod tests { struct Inner { ready: bool } }
