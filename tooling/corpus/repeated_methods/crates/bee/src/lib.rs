pub struct Bee;
impl Bee { pub fn walk(&self, steps: Steps) -> Steps { steps } pub fn new() -> Self { Bee } fn hide(&self, steps: Steps) {} }
impl Walk for Bee { fn rest(&self) {} }
impl Bee { pub fn hold<T: Into<Steps>>(&self, load: T) -> Steps { load.into() } }
