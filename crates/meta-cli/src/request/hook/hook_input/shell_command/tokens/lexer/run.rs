mod depth;
mod word;

use depth::Depth;
use word::Word;

use super::bracket::Bracket;
use super::closer::Closer;
use super::mode::Mode;

#[derive(Default)]
pub struct Run {
    word: Word,
    segment: Vec<String>,
    mode: Mode,
    closer: Option<Closer>,
    depth: Depth,
}

impl Run {
    pub fn inside(closer: Closer) -> Self {
        Self { closer: Some(closer), ..Self::default() }
    }

    pub fn restored(outer: Self) -> Self {
        Self { word: outer.word.started(), ..outer }
    }

    pub fn closes(&self, c: char) -> bool {
        self.closer.is_some_and(|closer| closer.symbol() == c) && self.depth.is_top()
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn word(&self) -> &Word {
        &self.word
    }

    pub fn segment(&self) -> &[String] {
        &self.segment
    }

    pub fn pushed(self, c: char) -> Self {
        Self { word: self.word.pushed(c), ..self }
    }

    pub fn word_ended(self) -> Self {
        let mut segment = self.segment;
        segment.extend(self.word.into_text());
        Self { word: Word::default(), segment, ..self }
    }

    pub fn with_word(self, text: String) -> Self {
        let mut segment = self.segment;
        segment.push(text);
        Self { segment, ..self }
    }

    pub fn moded(self, mode: Mode) -> Self {
        let word = match mode {
            Mode::Single | Mode::Double => self.word.started(),
            Mode::Plain | Mode::Comment => self.word,
        };
        Self { word, mode, ..self }
    }

    pub fn separated(self, bracket: Option<Bracket>) -> Self {
        Self { depth: self.depth.after(bracket), segment: Vec::new(), ..self }
    }
}
