pub struct Word; impl core::str::FromStr for Word { type Err = (); fn from_str(text: &str) -> Result<Word, ()> { text.trim().parse() } } //~ primitive-boundary: trim handles text outside a door
impl Word { fn shout(&self) -> String { format!("{self}") } } //~ primitive-boundary: format handles text outside a door
