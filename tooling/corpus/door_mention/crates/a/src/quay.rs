pub struct Quay; impl core::str::FromStr for Quay { type Err = (); fn from_str(_: &str) -> Result<Quay, ()> { Ok(Quay) } }
impl Quay { pub fn tidy(text: &str) -> &str { text.trim() } }
pub struct Harbor; impl Harbor { pub fn opened(&self) { custom::register!(Quay); } }
