use super::Tokens;

pub struct Scanner {
    chars: Vec<char>,
    at: usize,
    word: Option<String>,
    segment: Vec<String>,
    found: Tokens,
}

impl Scanner {
    pub fn tokens(text: &str) -> Tokens {
        let mut scanner =
            Self { chars: text.chars().collect(), at: 0, word: None, segment: Vec::new(), found: Tokens::default() };
        scanner.run(None);
        scanner.found
    }

    fn run(&mut self, end: Option<char>) {
        let mut depth = 0usize;
        while let Some(c) = self.next() {
            if Some(c) == end && depth == 0 {
                break;
            }
            match c {
                '\'' => self.single(),
                '"' => self.double(),
                '\\' => self.escaped(),
                '$' if self.peek(0) == Some('(') => self.substitution(),
                '`' => self.nested('`'),
                '#' if self.word.is_none() => self.comment(),
                '\n' | ';' | '&' | '|' => self.end_segment(),
                '(' | ')' => {
                    depth = if c == '(' { depth + 1 } else { depth.saturating_sub(1) };
                    self.end_segment();
                    self.found.segments.push(vec![c.to_string()]);
                }
                '>' => self.output(),
                '<' => self.input(),
                c if c.is_whitespace() => self.end_word(),
                c => self.push(c),
            }
        }
        self.end_segment();
    }

    fn next(&mut self) -> Option<char> {
        let c = self.peek(0);
        self.at += usize::from(c.is_some());
        c
    }

    fn peek(&self, ahead: usize) -> Option<char> {
        self.chars.get(self.at + ahead).copied()
    }

    fn push(&mut self, c: char) {
        self.word.get_or_insert_default().push(c);
    }

    fn end_word(&mut self) {
        let word = self.word.take();
        self.segment.extend(word);
    }

    fn end_segment(&mut self) {
        self.end_word();
        let segment = std::mem::take(&mut self.segment);
        if !segment.is_empty() {
            self.found.segments.push(segment);
        }
    }

    fn nested(&mut self, end: char) {
        let saved = (self.word.take(), std::mem::take(&mut self.segment));
        self.run(Some(end));
        (self.word, self.segment) = saved;
        self.word.get_or_insert_default();
    }

    fn substitution(&mut self) {
        self.at += 1;
        self.nested(')');
    }

    fn single(&mut self) {
        self.word.get_or_insert_default();
        while let Some(c) = self.next().filter(|c| *c != '\'') {
            self.push(c);
        }
    }

    fn double(&mut self) {
        self.word.get_or_insert_default();
        while let Some(c) = self.next().filter(|c| *c != '"') {
            match c {
                '\\' => self.escaped(),
                '$' if self.peek(0) == Some('(') => self.substitution(),
                '`' => self.nested('`'),
                c => self.push(c),
            }
        }
    }

    fn escaped(&mut self) {
        if let Some(c) = self.next().filter(|c| *c != '\n') {
            self.push(c);
        }
    }

    fn comment(&mut self) {
        while self.peek(0).is_some_and(|c| c != '\n') {
            self.at += 1;
        }
    }

    fn output(&mut self) {
        match (self.peek(0), self.peek(1)) {
            (Some('('), _) => {}
            (Some('&'), Some(fd)) if fd.is_ascii_digit() || fd == '-' => self.at += 2,
            _ => self.found.redirects = true,
        }
    }

    fn input(&mut self) {
        match (self.peek(0), self.peek(1)) {
            (Some('<'), Some('<')) => self.at += 2,
            (Some('<'), _) => {
                self.at += 1;
                self.found.heredoc |= Self::delimited(self.chars.get(self.at..).unwrap_or_default());
            }
            _ => {}
        }
    }

    fn delimited(tail: &[char]) -> bool {
        let first = tail.iter().skip_while(|c| **c == '-').find(|c| !c.is_whitespace());
        first.is_some_and(|c| c.is_alphabetic() || matches!(c, '_' | '\'' | '"' | '\\'))
    }
}
