use super::super::super::kind::Kind;

#[derive(Default)]
pub struct Word {
    text: Option<String>,
    kind: Kind,
}

impl Word {
    pub fn pushed(self, c: char) -> Self {
        let mut text = self.text.unwrap_or_default();
        text.push(c);
        Self { text: Some(text), ..self }
    }

    pub fn started(self) -> Self {
        Self { text: Some(self.text.unwrap_or_default()), ..self }
    }

    pub fn marked(self, kind: Kind) -> Self {
        Self { kind, ..self }
    }

    pub fn into_text(self) -> Option<String> {
        self.text
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn unstarted(&self) -> bool {
        self.text.is_none()
    }
}
