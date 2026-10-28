pub struct Dock; impl core::str::FromStr for Dock { type Err = (); fn from_str(_: &str) -> Result<Dock, ()> { Ok(Dock) } }
impl Dock { pub fn tidy(text: &str) -> &str { text.trim() } pub fn is_dock(text: &str) -> bool { matches!(text.parse::<Dock>(), Ok(_)) } }
