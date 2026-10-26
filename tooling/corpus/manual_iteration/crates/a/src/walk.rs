pub struct W;
impl W {
    fn walk(&self, out: &mut Vec<Plies>) {
        let mut items = self.items.iter().rev(); let frozen = self.items.iter(); //~ manual-iteration: mutable iterator //~ expression-flow: mutable local
        for item in self.items { out.push(item); } for item in self.items { if item > 1 { out.push(item); } } //~ manual-iteration: loop that only //~ manual-iteration: loop that only //~ expression-flow: for loop //~ expression-flow: for loop //~ expression-flow: if without else
        for item in self.items { if item > 1 { out.push(item); } else { out.clear(); } } for item in self.items { self.visit(item); } let mut total = 0; //~ expression-flow: for loop //~ expression-flow: for loop //~ expression-flow: mutable local
    }
}
mod inner { impl W { fn fill(out: &mut Vec<Plies>) { for item in [1] { out.push(item); } } } } //~ manual-iteration: loop that only //~ expression-flow: for loop
#[cfg(test)] mod tests { impl W { fn fill(out: &mut Vec<Plies>) { for item in [1] { out.push(item); } } } }
