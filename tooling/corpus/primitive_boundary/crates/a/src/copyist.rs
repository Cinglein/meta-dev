pub struct Copyist; impl Clone for Copyist { fn clone(&self) -> Copyist { Copyist } }
impl Copyist { pub fn tidy(text: &str) -> &str { text.trim() } } //~ primitive-boundary: trim handles text outside a door
