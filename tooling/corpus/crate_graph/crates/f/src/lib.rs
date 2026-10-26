mod port; //~ crate crate-graph: conductance 0.067 between 9 items [ANCHOR Axle Cart Dock Hub Quay Rope Spoke Wheel] and 8 items [Deck FLAG Helm Hull Keel Mast Sail launch]
pub struct Sail(Mast);
pub enum Mast { Up(Box<dyn Keel>) }
pub trait Keel { fn hull(&self) -> Hull; }
pub type Hull = crate::Deck;
pub struct Deck(inner::Helm);
mod inner { pub struct Helm(super::Rope); }
pub struct Rope;
pub static ANCHOR: Rope = Rope;
pub const FLAG: Option<Sail> = None;
pub fn launch() -> Sail { todo!() } //~ no-free-fns: fn launch
impl Rope { pub fn tie(&self) -> Cart { todo!() } }
pub struct Cart(Wheel);
pub struct Wheel(Spoke);
pub struct Spoke(Axle);
pub struct Axle(Hub);
pub struct Hub(port::Dock);
#[cfg(test)] pub struct Probe(Sail, Cart);
