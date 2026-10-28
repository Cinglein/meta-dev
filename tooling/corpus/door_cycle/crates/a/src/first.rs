pub struct First; impl core::str::FromStr for First { type Err = (); fn from_str(text: &str) -> Result<First, ()> { Second::from_str(text).map(|_| First) } }
impl First { pub fn tidy(text: &str) -> &str { text.trim() } } //~ primitive-boundary: trim handles text outside a door
