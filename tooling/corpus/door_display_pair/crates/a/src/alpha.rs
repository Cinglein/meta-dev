pub struct Alpha; impl core::fmt::Display for Alpha { fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(formatter, "{}", "alpha") } }
impl Alpha { pub fn tidy(text: &str) -> &str { text.trim() } } //~ primitive-boundary: trim handles text outside a door
