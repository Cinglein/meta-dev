pub struct Wharf; impl core::str::FromStr for Wharf { type Err = (); fn from_str(_: &str) -> Result<Wharf, ()> { Ok(Wharf) } }
impl Wharf { pub fn tidy(text: &str) -> &str { text.trim() } } //~ primitive-boundary: trim handles text outside a door
