pub struct Bay; impl TryFrom<u8> for Bay { type Error = (); fn try_from(_: u8) -> Result<Bay, ()> { Ok(Bay) } }
impl core::str::FromStr for Bay { type Err = (); fn from_str(_: &str) -> Result<Bay, ()> { Ok(Bay) } }
