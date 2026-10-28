pub struct Port; impl core::fmt::Display for Port { fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { formatter.write_str("port") } }
impl Port { pub fn tidy(text: &str) -> &str { text.trim() } pub fn logged(&self) { tracing::info!(port = %self); } }
