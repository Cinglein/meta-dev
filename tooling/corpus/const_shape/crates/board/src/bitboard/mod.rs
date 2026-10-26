pub struct Bitboard(u64);
impl Bitboard { fn widen(x: u8) -> u16 { x as u16 } } //~ const-shape: as cast outside a const fn or a bit boundary file
