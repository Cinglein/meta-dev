pub struct Ant;
impl Ant { pub fn walk(&self, steps: Steps) -> Steps { steps } } //~ repeated-methods: fn walk(& self, Steps) -> Steps on Ant, Bee, Cow
impl Ant { pub fn new() -> Ant { Ant } fn hide(&self, steps: Steps) {} pub fn rest(&self) {} } //~ repeated-methods: fn new() -> Self on Ant, Bee, Cow
impl Ant { pub fn hold<T: Into<Steps>>(&self, load: T) -> Steps { load.into() } } //~ repeated-methods: fn hold< T : Into < Steps > >(& self, T) -> Steps on Ant, Bee, Cow
