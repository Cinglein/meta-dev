pub struct Word; impl core::str::FromStr for Word { type Err = (); fn from_str(text: &str) -> Result<Word, ()> { text.trim().parse() } } impl Word { fn shout(&self) -> String { format!("{self}") } }
