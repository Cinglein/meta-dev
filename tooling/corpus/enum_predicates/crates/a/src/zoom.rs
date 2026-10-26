pub enum Zoom { Day, Leg }
impl Zoom {
    pub fn is_leg(self) -> bool { matches!(self, Zoom::Leg) } //~ enum-predicates: fn is_leg -> bool
    pub fn other(self) -> Zoom { self }
}
impl PartialEq for Zoom { fn eq(&self, other: &Zoom) -> bool { true } }
pub struct Plain;
impl Plain { pub fn is_plain(&self) -> bool { true } }
