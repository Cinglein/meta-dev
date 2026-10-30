#[derive(Default)]
pub struct Word(Option<String>);

impl Word {
    pub fn pushed(self, c: char) -> Self {
        let mut text = self.0.unwrap_or_default();
        text.push(c);
        Self(Some(text))
    }

    pub fn started(self) -> Self {
        Self(Some(self.0.unwrap_or_default()))
    }

    pub fn into_text(self) -> Option<String> {
        self.0
    }

    pub fn unstarted(&self) -> bool {
        self.0.is_none()
    }
}
