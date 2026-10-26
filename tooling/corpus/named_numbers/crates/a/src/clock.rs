pub struct Clock;
impl Clock { const HALF: Ticks = 500; pub fn halved(ticks: Ticks) -> Ticks { ticks / 2 + 1 } } //~ named-numbers: numeric literal outside a const item
impl Clock { pub fn scaled(ticks: Speed) -> Speed { ticks * 0.5 } pub fn named(ticks: Ticks) -> Halves { ticks / Self::HALF } pub fn unit(ticks: Ticks) -> Rest { ticks - 1 } } //~ named-numbers: numeric literal outside a const item
#[cfg(any(test, feature = "proptest"))] impl Clock { pub fn range() -> Ticks { 60 } }
#[cfg(feature = "metal")] impl Clock { pub fn lanes() -> Lanes { 8 } } //~ named-numbers: numeric literal outside a const item
pub trait Rate { const BASE: Scale = 2.5; fn beat(&self) -> Ticks { 3 } } const LIMIT: Ticks = 40; //~ named-numbers: numeric literal outside a const item
pub fn capped(ticks: Ticks) -> Ticks { let edge = const { 7 }; assert!(ticks > 9); ticks.min(LIMIT) } //~ named-numbers: numeric literal outside a const item //~ no-free-fns: fn capped
mod inner { impl Clock { pub fn depth() -> Depth { 4 } } } //~ named-numbers: numeric literal outside a const item
#[cfg(test)] mod tests { impl Clock { fn depth() -> Depth { 4 } } }
