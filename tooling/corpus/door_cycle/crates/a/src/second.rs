pub struct Second; impl core::str::FromStr for Second { type Err = (); fn from_str(text: &str) -> Result<Second, ()> { First::from_str(text).map(|_| Second) } }
impl Second { pub fn tidy(text: &str) -> &str { text.trim() } } //~ primitive-boundary: trim handles text outside a door
