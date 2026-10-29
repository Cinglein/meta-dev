use std::borrow::Cow;
use std::io::Read;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub struct Stdout(Result<String, FromUtf8Error>);

impl Stdout {
    pub(crate) fn read_from(pipe: Option<impl Read>) -> Self {
        let mut bytes = Vec::new();
        pipe.map(|mut pipe| pipe.read_to_end(&mut bytes).ok());
        Self(String::from_utf8(bytes))
    }

    #[must_use]
    pub fn text(&self) -> Cow<'_, str> {
        match &self.0 {
            Ok(text) => Cow::Borrowed(text),
            Err(unreadable) => String::from_utf8_lossy(unreadable.as_bytes()),
        }
    }
}
