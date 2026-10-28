pub struct Bound; impl core::str::FromStr for Bound { type Err = (); fn from_str(_: &str) -> Result<Bound, ()> { Ok(Bound) } }
impl Bound { pub fn tidy(text: &str) -> &str { text.trim() } }
