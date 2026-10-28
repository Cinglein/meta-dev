pub struct Gate; impl core::fmt::Display for Gate { fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { formatter.write_str("gate") } }
impl Gate { pub fn label(&self) -> Option<String> { Some(self.to_string()).filter(|text| !text.is_empty()) } }
