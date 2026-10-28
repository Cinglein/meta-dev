use crate::port::Port as Harbour; pub struct Pier; impl Pier { pub fn moor() -> Option<Harbour> { Harbour::from_str("pier").ok() } }
