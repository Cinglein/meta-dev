pub struct Anchor; impl core::fmt::Display for Anchor { fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(formatter, "{}", "anchor") } }
impl Anchor { pub fn tidy(text: &str) -> &str { text.trim() } } //~ primitive-boundary: trim handles text outside a door
