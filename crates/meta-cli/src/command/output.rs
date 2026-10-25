use std::fmt;

use meta_schema::Record;

#[derive(Debug)]
pub enum Output {
    Written(Record),
    Listed(Listed),
}

#[derive(Debug)]
pub struct Listed(String);

impl Listed {
    pub fn new(text: impl fmt::Display) -> Self {
        Self(text.to_string())
    }
}

impl fmt::Display for Output {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Written(record) => writeln!(formatter, "{record}"),
            Self::Listed(listed) => formatter.write_str(&listed.0),
        }
    }
}
