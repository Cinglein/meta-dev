pub struct Port; impl core::str::FromStr for Port { type Err = (); fn from_str(_: &str) -> Result<Port, ()> { Ok(Port) } }
impl Port { pub fn tidy(text: &str) -> &str { text.trim() } pub fn is_ready(&self) -> bool { true } }
pub struct Registry; impl Registry { pub fn port(&self) -> Option<Port> { self.get::<Port>().filter(Port::is_ready) } }
