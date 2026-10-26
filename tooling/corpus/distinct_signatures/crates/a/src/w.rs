struct W; //~ file distinct-signatures: alpha, beta, gamma share the signature W::(self) -> Plies //~ file distinct-signatures: one, two share the signature V::(&self)
struct Plies;
impl W { fn alpha(self) -> Plies { Plies } fn beta(self) -> Plies { Plies } pub fn gamma(self) -> Plies { Plies } }
impl W { fn delta(&self) -> Plies { Plies } pub(crate) fn epsilon(self, other: Plies) -> Plies { other } }
mod inner { struct V; impl V { fn one(&self) {} fn two(&self) {} } }
#[cfg(test)] mod tests { struct T; impl T { fn one(&self) {} fn two(&self) {} } }
