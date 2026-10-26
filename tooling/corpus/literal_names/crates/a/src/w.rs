struct W;
impl W { fn of(x: Plies) -> W { W } fn check(&self) {} pub fn new() -> W { W } pub fn least_significant_bit(&self) -> Plies { Plies } } //~ literal-names: fn of names nothing //~ literal-names: fn check names nothing
impl From<Plies> for W { fn from(x: Plies) -> W { W } } trait T { fn do_it(&self); fn go(&self); } //~ literal-names: fn go names nothing
fn process() {} //~ literal-names: fn process names nothing //~ no-free-fns: fn process
mod inner { impl W { fn handle() {} } } //~ literal-names: fn handle names nothing
#[cfg(test)] mod tests { impl W { fn make() {} } }
