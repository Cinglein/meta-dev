pub struct Buoy; impl core::fmt::Display for Buoy { fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(formatter, "{}", "buoy") } }
impl Buoy { pub fn tidy(text: &str) -> &str { text.trim() } } //~ primitive-boundary: trim handles text outside a door
