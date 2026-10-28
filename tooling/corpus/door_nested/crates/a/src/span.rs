pub struct Span; impl core::str::FromStr for Span { type Err = (); fn from_str(text: &str) -> Result<Span, ()> { Bound::from_str(text).map(|_| Span) } }
