use std::fmt;
use std::io::Read;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stderr(String);

impl Stderr {
    pub(crate) fn read_from(pipe: Option<impl Read>) -> Self {
        let mut bytes = Vec::new();
        pipe.map(|mut pipe| pipe.read_to_end(&mut bytes).ok());
        Self::said(String::from_utf8_lossy(&bytes).trim().to_owned())
    }

    #[must_use]
    pub fn said(text: String) -> Self {
        Self(text)
    }

    #[must_use]
    pub fn first_line(&self) -> Option<&str> {
        self.0.lines().next()
    }
}

impl fmt::Display for Stderr {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
