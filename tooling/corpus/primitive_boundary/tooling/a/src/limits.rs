pub struct Limits { nodes: u64, index: usize, rounds: usize } pub enum Kind { Deep(u8) } //~ primitive-boundary: bare u64 in nodes //~ primitive-boundary: bare u8 in Kind //~ primitive-boundary: usize in rounds
impl Limits { const CAP: u32 = 1; fn read(&self) -> Option<u64> { let x: u8 = 1; self.nodes.into() } } //~ primitive-boundary: bare u32 in CAP //~ primitive-boundary: bare u64 in read
const LIMIT: u64 = 1; pub trait Bound { const MOST: u8; } pub type Count = u32; //~ primitive-boundary: bare u64 in LIMIT //~ primitive-boundary: bare u8 in MOST //~ primitive-boundary: bare u32 in Count
mod inner { impl Limits { pub fn depth() -> u16 { 1 } } } //~ primitive-boundary: bare u16 in depth
#[cfg(test)] mod tests { impl Limits { pub fn depth() -> u16 { 1 } } }
