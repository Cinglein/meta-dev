#[derive(serde::Deserialize)] pub struct Plan { name: String, size: Option<String> }
impl Plan { pub fn is_sized(text: &str) -> bool { text.parse::<u32>().is_ok() && u8::try_from(1).is_ok() } pub fn shown(&self) { println!("{self:?} {:?} {:#?}", self.name, self.size); tracing::debug!(count = 1); } }
