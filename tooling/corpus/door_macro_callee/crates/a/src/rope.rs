pub struct Rope; impl TryFrom<u8> for Rope { type Error = (); fn try_from(_: u8) -> Result<Rope, ()> { Ok(Rope) } }
impl Rope { pub fn tidy(text: &str) -> &str { text.trim() } pub fn is_rope(text: &str) -> bool { custom::check!(text.parse()) } }
