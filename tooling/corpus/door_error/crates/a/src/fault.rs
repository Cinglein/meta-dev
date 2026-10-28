pub struct Fault; impl core::fmt::Display for Fault { fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { formatter.write_str("fault") } }
impl Fault { pub fn tidy(text: &str) -> &str { text.trim() } }
#[derive(Debug)] #[error("failed: {0}")] pub struct Failed(Fault);
