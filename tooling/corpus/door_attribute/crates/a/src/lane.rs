#[serde(try_from = "String")] pub struct Lane; impl TryFrom<String> for Lane { type Error = (); fn try_from(_: String) -> Result<Lane, ()> { Ok(Lane) } }
impl Lane { pub fn tidy(text: &str) -> &str { text.trim() } }
