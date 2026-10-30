mod depth;
mod word;

use depth::Depth;
use word::Word;

use super::super::kind::Kind;
use super::super::segment::Segment;
use super::bracket::Bracket;
use super::closer::Closer;
use super::mode::Mode;

#[derive(Default)]
pub struct Run {
    word: Word,
    segment: Segment,
    mode: Mode,
    closer: Option<Closer>,
    depth: Depth,
}

impl Run {
    const CONDITIONAL: &'static str = "[[";

    pub fn inside(closer: Closer) -> Self {
        Self { closer: Some(closer), ..Self::default() }
    }

    pub fn restored(outer: Self) -> Self {
        Self { word: outer.word.started(), ..outer }
    }

    pub fn closes(&self, c: char) -> bool {
        self.closer.is_some_and(|closer| closer.symbol() == c) && self.depth.is_top()
    }

    pub fn conditional(&self) -> bool {
        self.segment.words().first().is_some_and(|word| word == Self::CONDITIONAL)
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn word(&self) -> &Word {
        &self.word
    }

    pub fn segment(&self) -> &Segment {
        &self.segment
    }

    pub fn pushed(self, c: char) -> Self {
        Self { word: self.word.pushed(c), ..self }
    }

    pub fn marked(self, kind: Kind) -> Self {
        Self { word: self.word.marked(kind), ..self }
    }

    pub fn word_ended(self) -> Self {
        let kind = self.word.kind();
        let segment = match self.word.into_text() {
            Some(text) => self.segment.appended(text, kind),
            None => self.segment,
        };
        Self { word: Word::default(), segment, ..self }
    }

    pub fn moded(self, mode: Mode) -> Self {
        let word = match mode {
            Mode::Single | Mode::Double | Mode::ArithmeticPlain | Mode::ArithmeticDouble => self.word.started(),
            Mode::Plain | Mode::Comment => self.word,
        };
        Self { word, mode, ..self }
    }

    pub fn separated(self, bracket: Option<Bracket>) -> Self {
        Self { depth: self.depth.after(bracket), segment: Segment::default(), ..self }
    }
}
