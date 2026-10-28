pub struct Port; impl core::str::FromStr for Port { type Err = (); fn from_str(_: &str) -> Result<Port, ()> { Ok(Port) } }
impl Port { pub fn tidy(text: &str) -> &str { text.trim() } }
