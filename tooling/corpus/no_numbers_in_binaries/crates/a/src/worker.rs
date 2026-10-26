pub struct W;
impl W { pub fn size() -> usize { 42 } pub fn rate() -> Rate { 2.5 } } //~ no-numbers-in-binaries: numeric literal in a binary //~ no-numbers-in-binaries: numeric literal in a binary //~ named-numbers: numeric literal outside a const item //~ named-numbers: numeric literal outside a const item
mod inner { impl W { pub fn depth() -> Depth { 7 } } } //~ no-numbers-in-binaries: numeric literal in a binary //~ named-numbers: numeric literal outside a const item
#[cfg(test)] mod tests { impl W { fn depth() -> Depth { 7 } } }
