struct Own; enum Mode { Fast } trait Rule { fn apply(x: Plies) -> Plies; } //~ file private-fns: W has 19 private fns
struct W { inner: Own, raw: u64 } //~ primitive-boundary: bare u64 in raw
impl W {
    fn to_own(&self) -> Weight { self.inner.value() } fn to_foreign(&self) -> Ones { self.raw.count_ones() } //~ no-forwarders: fn to_own only forwards to value
    fn to_self(self, other: W) -> W { self.merge(other) } fn to_path(bits: Bits) -> W { W::assemble(bits) } //~ no-forwarders: fn to_self only forwards to merge //~ no-forwarders: fn to_path only forwards to W::assemble
    fn to_self_path(bits: Bits) -> Assembled { Self::assemble(bits) } fn to_std(bits: u64) -> u64 { u64::from(bits) } //~ no-forwarders: fn to_self_path only forwards to W //~ primitive-boundary: bare u64 in to_std //~ primitive-boundary: bare u64 in to_std
    fn to_param(x: &Own) -> Weight { x.value() } fn to_foreign_param(x: &u64) -> Ones { x.count_ones() } //~ no-forwarders: fn to_param only forwards to value //~ primitive-boundary: bare u64 in to_foreign_param
    fn to_enum(mode: Mode) -> Rank { mode.rank() } fn to_trait(x: Plies) -> Plies { Rule::apply(x) } //~ no-forwarders: fn to_enum only forwards to rank //~ no-forwarders: fn to_trait only forwards to Rule::apply
    fn passes_field(&self, other: W) -> Weight { self.inner.weigh(other.raw) } //~ no-forwarders: fn passes_field only forwards to weigh
    fn passes_reference(&self, other: W) -> Mass { self.inner.weigh(&other) } //~ no-forwarders: fn passes_reference only forwards to weigh
    fn constructs(bits: Bits, mode: Mode) -> W { W(bits, mode) } fn fixes_a_call() -> W { W::assemble(0) }
    fn negates(self) -> Negated { -self.merge(self) } fn fixes_an_argument(self) -> W { self.merge(W::EMPTY) } fn to_other_field(other: W) -> Weight { other.inner.value() }
}
struct P<H> { hand: H } impl P<Own> { fn to_generic(&self) -> Weight { self.hand.value() } } //~ no-forwarders: fn to_generic only forwards to value
impl From<u64> for W { fn from(bits: u64) -> W { W::to_path(bits) } }
mod inner { impl W { fn to_inner(&self) -> Inner { self.inner.value() } } } //~ no-forwarders: fn to_inner only forwards to value
#[cfg(test)] mod tests { impl W { fn to_test(&self) -> Weight { self.inner.value() } } }
