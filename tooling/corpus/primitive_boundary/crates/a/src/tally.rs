pub struct Tally; impl core::str::FromStr for Tally { type Err = (); fn from_str(text: &str) -> Result<Tally, ()> { text.trim().is_empty().then_some(Tally).ok_or(()) } }
impl Tally { pub fn read(text: &str) -> Option<Tally> { text.trim().parse::<Tally>().ok() } }
