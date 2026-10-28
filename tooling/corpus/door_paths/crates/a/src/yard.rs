pub struct Yard; impl TryFrom<u8> for Yard { type Error = (); fn try_from(_: u8) -> Result<Yard, ()> { Ok(Yard) } }
impl Yard { pub fn first() -> Option<Yard> { Yard::try_from(1).ok() } pub fn tidy(text: &str) -> &str { text.trim() } }
