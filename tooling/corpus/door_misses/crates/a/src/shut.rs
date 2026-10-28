#[derive(Debug, Clone)] #[allow(dead_code)] pub struct Shut; impl TryFrom<u8> for Shut { type Error = (); fn try_from(_: u8) -> Result<Shut, ()> { Ok(Shut) } }
impl core::fmt::Display for Shut { fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { formatter.write_str("shut") } }
impl core::str::FromStr for Shut { type Err = (); fn from_str(_: &str) -> Result<Shut, ()> { Ok(Shut) } }
impl Shut { pub fn tidy(text: &str) -> &str { text.trim() } pub fn is_wide() -> bool { Bay::try_from(1).is_ok() && str::parse::<Bay>("1").is_ok() } pub fn every() -> Vec<Shut> { vec![Shut] } #[cfg(Shut)] pub fn gated() {} } //~ primitive-boundary: trim handles text outside a door
