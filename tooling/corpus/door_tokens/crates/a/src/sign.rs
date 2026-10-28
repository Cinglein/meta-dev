pub struct Sign; impl core::fmt::Display for Sign { fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { formatter.write_str("sign") } }
impl Sign { pub fn shown(&self) -> String { format!("[{self}]") } }
