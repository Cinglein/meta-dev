pub struct Flow;
impl Flow {
    fn guards(&self, items: &[Plies]) -> Plies {
        if items.is_empty() { return 0; } for item in items { continue; } //~ expression-flow: if or match //~ expression-flow: if without else //~ expression-flow: early return //~ expression-flow: for loop //~ expression-flow: continue
        match items.len() { 0 => self.note(0), _ => self.note(1) }; let mut total = 0; total += items.len(); let plain = 0; //~ expression-flow: mutable local //~ expression-flow: if or match //~ expression-flow: compound assignment
        let mut sorted = items.to_vec(); sorted.sort(); let mut buffer = Vec::new(); self.fill(&mut buffer);
        let mut pair = Pair::default(); pair.left = 1; let mut slots = [0; 1]; slots[0] = 1; //~ expression-flow: mutable local //~ expression-flow: mutable local //~ expression-flow: assignment to //~ expression-flow: assignment to
        let mut count = 0; (count) = 1; while self.busy() { break; } //~ expression-flow: mutable local //~ expression-flow: assignment to //~ expression-flow: while loop //~ expression-flow: break
        if items.is_empty() { self.note(0) } else { self.note(1) } self.note(&total); //~ expression-flow: if or match
        if total > 1 { self.note(0) } else { self.note(1) }
    }
    const fn walk(mut rest: &[Plies]) -> Plies { while let [head, tail @ ..] = rest { rest = tail; } 0 }
}
fn free() -> Plies { loop { break 1; } } trait Step { fn once(&self) -> Plies { return 1; } } //~ expression-flow: loop //~ expression-flow: break //~ expression-flow: early return //~ no-free-fns: fn free
mod inner { fn again() -> Plies { return 1; } } //~ expression-flow: early return //~ no-free-fns: fn again
#[cfg(test)] mod tests { impl Flow { fn again() -> Plies { return 1; } } }
