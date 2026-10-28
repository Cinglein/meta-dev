pub struct Gauge; impl core::str::FromStr for Gauge { type Err = (); fn from_str(_: &str) -> Result<Gauge, ()> { Ok(Gauge) } }
impl Gauge { pub fn tidy(text: &str) -> &str { text.trim() } }
#[derive(serde::Serialize)] pub enum Reading { Gauge(Gauge) }
