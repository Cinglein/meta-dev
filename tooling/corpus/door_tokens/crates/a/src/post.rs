#[serde(try_from = "u8")] pub struct Post; impl TryFrom<u8> for Post { type Error = (); fn try_from(_: u8) -> Result<Post, ()> { Ok(Post) } }
impl Post { pub fn tidy(text: &str) -> &str { text.trim() } }
