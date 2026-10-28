pub struct Berth; impl TryFrom<u8> for Berth { type Error = (); fn try_from(_: u8) -> Result<Berth, ()> { Ok(Berth) } }
impl Berth { pub fn tidy(text: &str) -> &str { text.trim() } }
