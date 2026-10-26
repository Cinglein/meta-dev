pub struct Hub; impl State for Hub {} pub struct Spoke; impl State for Spoke {} //~ workspace state-graph: Hub -> Spoke has 3 edges, at most one //~ file distinct-signatures: hidden, first share the signature Hub::(self) -> Spoke //~ file distinct-signatures: peek, held share the signature Hub::(&self) -> Spoke
pub struct Rim; pub type Edge = Rim; impl State for Edge {}
pub enum Wire { Live(Volt), Dead, Faulty { code: Volt } } pub enum Side { Left, Right }
pub struct Leaky { pub inner: Volt } //~ state-graph: field inner of Leaky
impl Hub {
    fn hidden(self) -> Spoke { Spoke } fn hidden_rim(self) -> Rim { Rim } //~ state-graph: is a hidden edge Hub -> Spoke //~ state-graph: is a hidden edge Hub -> Rim
    pub fn first(self) -> Spoke { Spoke } pub fn again(self) -> Option<Spoke> { None } pub fn back(self) -> Hub { Hub }
    pub fn peek(&self) -> Spoke { Spoke } pub fn fresh() -> Spoke { Spoke } pub fn held(&self) -> Spoke { self.spoke } //~ state-graph: is a view returning vertex
    pub fn borrowed(&self, other: &Hub) -> Spoke { other.spoke } pub fn pair(self) -> (Spoke, Volt) { (Spoke, 0) } pub fn poke(&mut self) {} //~ state-graph: is a view returning vertex //~ state-graph: returns a tuple holding //~ state-graph: takes &mut self on vertex //~ type-shape: tuple in a signature
}
impl Spoke { pub fn home(self) -> Hub { Hub } }
impl Wire { fn is_live(&self) -> bool { match self { Wire::Live(_) => true, Wire::Dead => false } } }
