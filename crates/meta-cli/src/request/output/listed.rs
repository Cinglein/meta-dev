use std::fmt;

#[derive(Debug)]
pub struct Listed(String);

impl Listed {
    pub fn new(text: impl fmt::Display) -> Self {
        Self(text.to_string())
    }
}

impl fmt::Display for Listed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
