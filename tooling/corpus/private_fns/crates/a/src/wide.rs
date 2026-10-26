struct Wide; //~ file private-fns: Wide has 5 private fns, at most 4 allowed: one, two, four, five, seven
impl Wide { fn one(a: One) {} fn two(a: Two) {} pub fn three(a: Three) {} }
impl Wide { fn four(a: Four) {} fn five(a: Five) {} fn seven(a: Seven) {} pub(crate) fn six(a: Six) {} }
impl Iterator for Wide { type Item = Plies; fn next(&mut self) -> Option<Plies> { None } }
struct Narrow;
impl Narrow { fn one(a: One) {} fn two(a: Two) {} fn three(a: Three) {} fn four(a: Four) {} }
