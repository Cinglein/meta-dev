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
use super::kind::Kind;

#[derive(Default)]
pub struct Lexer {
    run: Run,
    saved: Vec<Run>,
    found: Tokens,
    skipping: Skip,
}

impl Lexer {
    const SEPARATOR: char = '\n';

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
            (Mode::ArithmeticPlain, [')', ')', tail @ ..]) => self.moded(Mode::Plain).skipped(Skip::past(rest, tail)),
            (Mode::ArithmeticDouble, [')', ')', tail @ ..]) => self.moded(Mode::Double).skipped(Skip::past(rest, tail)),
            (Mode::Comment, ['\n', ..]) => self.moded(Mode::Plain).read(Mode::Plain, rest),
            (Mode::Comment | Mode::ArithmeticPlain | Mode::ArithmeticDouble, _) | (_, []) => self,
            (Mode::Single, [c, ..]) => Self { run: self.run.pushed(*c), ..self },
            (_, ['\\', next, tail @ ..]) => self.escaped(Some(*next)).skipped(Skip::past(rest, tail)),
            (_, ['\\']) => self,
            (Mode::Double, ['$', '(', '(', tail @ ..]) => {
                self.arithmetic(Mode::ArithmeticDouble, Skip::past(rest, tail))
            }
            (_, ['$', '(', '(', tail @ ..]) => self.arithmetic(Mode::ArithmeticPlain, Skip::past(rest, tail)),
            (_, ['$', '(', tail @ ..]) => self.entered(Closer::Paren).skipped(Skip::past(rest, tail)),
            (_, ['`', ..]) => self.entered(Closer::Tick),
            (_, ['$', ..]) => Self { run: self.run.marked(Kind::Expanded).pushed('$'), ..self },
            (Mode::Double, [c, ..]) => Self { run: self.run.pushed(*c), ..self },
            (_, ['\'', ..]) => self.moded(Mode::Single),
            (_, ['"', ..]) => self.moded(Mode::Double),
            (_, ['#', ..]) if self.run.word().unstarted() => self.moded(Mode::Comment),
            (_, [c @ ('\n' | ';' | '&' | '|' | '(' | ')'), ..]) => self.separated(*c),
            (_, ['>', '(', ..]) => self,
            (_, ['>', '&', fd, tail @ ..]) if fd.is_ascii_digit() || *fd == '-' => self.skipped(Skip::past(rest, tail)),
            (_, ['>', '&', gap, tail @ ..]) if gap.is_whitespace() && Self::duplicates(tail) => {
                self.skipped(Skip::past(rest, tail))
            }
            (_, ['>', ..]) if self.run.conditional() => self,
            (_, ['>', ..]) => self.flagged(Some(Breach::Redirect)),
            (_, ['<', '<', '<', tail @ ..]) => self.flagged(Some(Breach::HereString)).skipped(Skip::past(rest, tail)),
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

    pub fn duplicates(tail: &[char]) -> bool {
        tail.iter().find(|c| !c.is_whitespace()).is_some_and(|c| c.is_ascii_digit() || *c == '-')
    }

    pub fn arithmetic(self, mode: Mode, skipping: Skip) -> Self {
        Self { run: self.run.marked(Kind::Expanded).moded(mode), ..self }.skipped(skipping)
    }

    pub fn entered(self, closer: Closer) -> Self {
        let mut saved = self.saved;
        saved.push(self.run.marked(Kind::Expanded));
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
        let found = self.found.extended(ended.segment().clone()).extended(alone);
        Self { run: ended.separated(bracket), found, ..self }
    }
}
