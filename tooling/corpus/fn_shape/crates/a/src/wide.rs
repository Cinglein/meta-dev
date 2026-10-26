pub struct Wide;
impl Wide { pub fn four(a: Plies, b: Plies, c: Plies, d: Plies) {}
    pub fn five(a: Plies, b: Plies, c: Plies, d: Plies, e: Plies) {} //~ fn-shape: more than 4
    fn flags(&self, x: Plies) {
        let ready = self.count() > 1; let (lo, hi) = (0, 1); //~ fn-shape: boolean local //~ fn-shape: tuple literal
        let total = self.items.iter().fold((0, 0), |(a, b), x| (a + x, b)); let plain = self.items.iter().fold(0, |a, x| a + x); //~ fn-shape: tuple accumulator
        let placed = self.place((0, 1)); let missing = !self.ready(); let inner = (x > 1); //~ fn-shape: boolean local //~ fn-shape: boolean local
        let empty = self.items.is_empty(); let owns = self.has_key(); //~ fn-shape: boolean local //~ fn-shape: boolean local
        while self.busy() { let idle = false; } loop { let (p, q) = (0, 1); } //~ fn-shape: boolean local //~ fn-shape: tuple literal //~ expression-flow: while loop //~ expression-flow: loop
        let sizes = self.items.iter().map(|y| { let big = y > 1; big }); //~ fn-shape: boolean local
        for y in self.items { if y > 1 { //~ expression-flow: for loop //~ expression-flow: if without else
            match y { 1 => self.go(), _ => {} } } } //~ fn-shape: control flow nested //~ fn-shape: control flow nested
    }
}
mod inner { impl Wide { fn deep(&self) { let flag = true; } } } //~ fn-shape: boolean local
#[cfg(test)] mod tests { impl Wide { fn deep(&self) { let flag = true; } } }
