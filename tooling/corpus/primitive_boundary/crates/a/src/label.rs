pub struct Label; impl core::fmt::Display for Label { fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { formatter.write_str("label") } }
impl Label { pub fn shown(&self) -> String { format!("[{self}]") } }
