pub struct Ask(Bid, Cap, Dip, Eel, Oak); //~ crate crate-graph: conductance 0.024 between 5 items
pub struct Bid(Ask, Cap, Dip, Eel);
pub struct Cap(Ask, Bid, Dip, Eel);
pub struct Dip(Ask, Bid, Cap, Eel);
pub struct Eel(Ask, Bid, Cap, Dip);
pub struct Oak(Pea, Rye, Sky, Tea);
pub struct Pea(Oak, Rye, Sky, Tea);
pub struct Rye(Oak, Pea, Sky, Tea);
pub struct Sky(Oak, Pea, Rye, Tea);
pub struct Tea(Oak, Pea, Rye, Sky);
impl Ask { pub fn run() { std::process::Pea::new(); } }
