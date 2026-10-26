pub trait Step { fn step(hub: Hub) -> Spoke; } pub struct Walker;
impl Step for Walker { fn step(hub: Hub) -> Spoke { Spoke } }
impl Walker {
    fn relay(hub: Hub) -> Spoke { Spoke } fn bump(&mut self) {} //~ state-graph: takes a vertex and returns
    pub fn label(wire: Wire, side: Side) -> Weight { match side { Side::Left => wire.weight(), Side::Right => 0 }; match wire { Wire::Live(n) => n.max(1), Wire::Dead => 0 } } //~ state-graph: match on Wire //~ expression-flow: if or match
    pub fn table(wire: Wire) -> Table { match wire { Wire::Live(_) => -(1), Wire::Dead => [(0, 1)] } }
    pub fn peel(wire: Wire) -> Peeled { if let Wire::Live(n) = wire { n } else { 0 } } //~ state-graph: match on Wire
    pub fn force(wire: Wire) -> Forced { let Wire::Live(n) = wire else { return 0 }; n } //~ state-graph: match on Wire //~ expression-flow: early return
    pub fn dead(wire: Wire) -> Weight { match wire { Wire::Dead => wire.weight(), _ => 0 } } //~ state-graph: match on Wire
    pub fn faulty(wire: Wire) -> Code { match wire { Wire::Faulty { code } => code.max(1), _ => 0 } } //~ state-graph: match on Wire
}
