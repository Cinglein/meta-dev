use std::fmt;

#[derive(Debug)]
pub struct XmlText<'text>(&'text str);

impl<'text> XmlText<'text> {
    pub fn new(text: &'text str) -> Self {
        Self(text)
    }
}

impl fmt::Display for XmlText<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.chars().try_for_each(|character| match character {
            '&' => formatter.write_str("&amp;"),
            '<' => formatter.write_str("&lt;"),
            '>' => formatter.write_str("&gt;"),
            other => write!(formatter, "{other}"),
        })
    }
}
