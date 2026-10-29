pub struct Cow;
impl Cow { pub fn walk(&self, steps: Steps) -> Steps { steps } pub fn new() -> Cow { Cow } fn hide(&self, steps: Steps) {} }
#[cfg(test)] impl Cow { pub fn rest(&self) {} }
impl Cow { pub fn hold<T: Into<Steps>>(&self, load: T) -> Steps { load.into() } }
mod pen { pub struct Elk; impl Elk { pub fn rest(&self) {} } #[cfg(test)] mod tests { impl Fox { pub fn rest(&self) {} } } }
