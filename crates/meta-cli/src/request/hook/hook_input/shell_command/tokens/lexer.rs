mod bracket;
mod closer;
mod mode;
mod run;
mod skip;

use bracket::Bracket;
use closer::Closer;
use mode::Mode;
use run::Run;
use skip::Skip;

use super::Tokens;
use super::breach::Breach;

#[derive(Default)]
pub struct Lexer {
    run: Run,
    saved: Vec<Run>,
    found: Tokens,
    skipping: Skip,
}

impl Lexer {
    const SEPARATOR: char = '\n';
    const HERE_STRING: &'static str = "<<<";

    pub fn stepped(self, rest: &[char]) -> Self {
        let mode = self.run.mode();
        match self.skipping.pending() {
            Some(skipping) => Self { skipping, ..self },
            None => self.read(mode, rest),
        }
    }

    pub fn finished(self) -> Tokens {
        let ended = self.separated(Self::SEPARATOR);
        if ended.saved.is_empty() { ended.found } else { ended.left().finished() }
    }

    pub fn left(self) -> Self {
        let ended = self.separated(Self::SEPARATOR);
        let mut saved = ended.saved;
        match saved.pop() {
            Some(outer) => Self { run: Run::restored(outer), saved, found: ended.found, skipping: ended.skipping },
            None => Self { run: ended.run, saved, found: ended.found, skipping: ended.skipping },
        }
    }

    pub fn read(self, mode: Mode, rest: &[char]) -> Self {
        match (mode, rest) {
            (Mode::Plain, [c, ..]) if self.run.closes(*c) => self.left(),
            (Mode::Single, ['\'', ..]) | (Mode::Double, ['"', ..]) => self.moded(Mode::Plain),
            (Mode::Comment, ['\n', ..]) => self.moded(Mode::Plain).read(Mode::Plain, rest),
            (Mode::Comment, _) | (_, []) => self,
            (Mode::Single, [c, ..]) => Self { run: self.run.pushed(*c), ..self },
            (_, ['\\', next, tail @ ..]) => self.escaped(Some(*next)).skipped(Skip::past(rest, tail)),
            (_, ['\\']) => self,
            (_, ['$', '(', tail @ ..]) => self.entered(Closer::Paren).skipped(Skip::past(rest, tail)),
            (_, ['`', ..]) => self.entered(Closer::Tick),
            (Mode::Double, [c, ..]) => Self { run: self.run.pushed(*c), ..self },
            (_, ['\'', ..]) => self.moded(Mode::Single),
            (_, ['"', ..]) => self.moded(Mode::Double),
            (_, ['#', ..]) if self.run.word().unstarted() => self.moded(Mode::Comment),
            (_, [c @ ('\n' | ';' | '&' | '|' | '(' | ')'), ..]) => self.separated(*c),
            (_, ['>', '(', ..]) => self,
            (_, ['>', '&', fd, tail @ ..]) if fd.is_ascii_digit() || *fd == '-' => self.skipped(Skip::past(rest, tail)),
            (_, ['>', ..]) => self.flagged(Some(Breach::Redirect)),
            (_, ['<', '<', '<', tail @ ..]) => {
                let run = self.run.word_ended().with_word(Self::HERE_STRING.into());
                Self { run, ..self }.skipped(Skip::past(rest, tail))
            }
            (_, ['<', '<', tail @ ..]) => self.flagged(Breach::heredoc(tail)).skipped(Skip::past(rest, tail)),
            (_, ['<', ..]) => self,
            (_, [c, ..]) if c.is_whitespace() => Self { run: self.run.word_ended(), ..self },
            (_, [c, ..]) => Self { run: self.run.pushed(*c), ..self },
        }
    }

    pub fn escaped(self, next: Option<char>) -> Self {
        match next.filter(|c| *c != Self::SEPARATOR) {
            Some(c) => Self { run: self.run.pushed(c), ..self },
            None => self,
        }
    }

    pub fn entered(self, closer: Closer) -> Self {
        let mut saved = self.saved;
        saved.push(self.run);
        Self { run: Run::inside(closer), saved, ..self }
    }

    pub fn moded(self, mode: Mode) -> Self {
        Self { run: self.run.moded(mode), ..self }
    }

    pub fn skipped(self, skipping: Skip) -> Self {
        Self { skipping, ..self }
    }

    pub fn flagged(self, breach: Option<Breach>) -> Self {
        Self { found: self.found.flagged(breach), ..self }
    }

    pub fn separated(self, c: char) -> Self {
        let bracket = Bracket::found_in(c);
        let ended = self.run.word_ended();
        let alone = bracket.map(Bracket::segment).unwrap_or_default();
        let found = self.found.extended(ended.segment().to_vec()).extended(alone);
        Self { run: ended.separated(bracket), found, ..self }
    }
}
