use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone, Copy, Debug)]
pub struct Reading<'table> {
    text: &'table str,
    shown: &'table str,
}

impl<'table> Reading<'table> {
    #[must_use]
    pub const fn new(text: &'table str, shown: &'table str) -> Self {
        Self { text, shown }
    }

    #[must_use]
    pub const fn verbatim(text: &'table str) -> Self {
        Self { text, shown: text }
    }

    #[must_use]
    pub fn text(self) -> &'table str {
        self.text
    }

    #[must_use]
    pub fn misread<Kind: FromStr + Display>(self) -> bool {
        self.text.parse::<Kind>().map(|read| ToString::to_string(&read)).ok().as_deref() != Some(self.shown)
    }
}
