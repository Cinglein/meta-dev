pub struct Ark(Bow); pub struct Bow(Cup); pub struct Cup(Dew); pub struct Dew(Elk); pub struct Elk(Fen); pub struct Fen; //~ crate crate-graph: conductance 0.000 between 7 items [Ark Bow Cup Dew Elk Fen Gap] and 5 items [Hob Inn Jar Keg Lid]
pub struct Gap; pub struct Hob; pub struct Inn; pub struct Jar; pub struct Keg; pub struct Lid;
#[cfg(test)] mod fixtures;
