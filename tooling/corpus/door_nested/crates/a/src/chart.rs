pub struct Chart; impl Chart { pub fn span(text: &str) -> Option<Span> { text.parse::<Span>().ok() } }
