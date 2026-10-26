pub struct Table(u8);
const CELLS: u8 = { let mut i = 0; while i < 1 { i += 1; } i }; //~ const-shape: counter loop
impl Table {
    const SIZE: u8 = { for n in 0..1 {} 0 }; //~ const-shape: for loop
    pub const fn free(mut rest: &[u8]) -> u8 { let mut i = 0; while i < 1 { i += 1; } while let [_, tail @ ..] = rest { rest = tail; } loop { break i } } //~ const-shape: counter loop //~ const-shape: open loop
    pub const fn assemble(key: u8) -> u8 { key as u8 }
    fn lookup(&self, key: Kind) -> u8 { let z = const { loop {} }; loop { break self.0[key as usize] } } //~ const-shape: open loop //~ const-shape: as cast //~ expression-flow: loop //~ expression-flow: loop //~ expression-flow: break
    pub fn counted(&self) -> u8 { let mut i = 0; while i < 1 { i += 1; } for n in 0..1 {} i } //~ expression-flow: mutable local //~ expression-flow: while loop //~ expression-flow: compound assignment //~ expression-flow: for loop
}
mod inner { impl Table { fn widen(x: u8) -> u16 { x as u16 } } } //~ const-shape: as cast
fn main() { let wide = 1u8 as u16; } //~ const-shape: as cast
#[cfg(test)] mod tests { impl Table { fn widen(x: u8) -> u16 { x as u16 } } }
