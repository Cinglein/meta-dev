pub struct Beta; impl core::fmt::Display for Beta { fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(formatter, "{}", "beta") } }
impl Beta { pub fn tidy(text: &str) -> &str { text.trim() } } //~ primitive-boundary: trim handles text outside a door
