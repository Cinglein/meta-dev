pub struct Seat;
pub struct Placer;
impl Placer {
    pub fn place(&self, taken: &mut Vec<Seat>) {} //~ vec-out-params: fn place
    pub fn read(&self, taken: &Vec<Seat>) {}
    pub fn own(&mut self, taken: Vec<Seat>) {}
}
#[cfg(test)] mod tests { impl Placer { fn fill(taken: &mut Vec<Seat>) {} } }
