#[derive(Clone, Copy, Debug, Default)]
pub struct Operators {
    redirects: bool,
    heredoc: bool,
}

impl Operators {
    const HEREDOC: &'static str = "shell rule: no heredocs; write the file with the Write tool";
    const REDIRECT: &'static str = "shell rule: no output redirection to a file, /dev/null included; write with the Write or Edit tool; pipes and reading are fine";

    pub fn scan(text: &str) -> Self {
        let chars: Vec<char> = text.chars().collect();
        let mut found = Self::default();
        let mut quote = None;
        let mut skip = 0;
        for (index, &c) in chars.iter().enumerate() {
            skip =
                if skip > 0 { skip - 1 } else { found.step(&mut quote, c, chars.get(index + 1..).unwrap_or_default()) };
        }
        found
    }

    pub fn breach(self) -> Option<&'static str> {
        self.heredoc.then_some(Self::HEREDOC).or(self.redirects.then_some(Self::REDIRECT))
    }

    fn step(&mut self, quote: &mut Option<char>, c: char, rest: &[char]) -> usize {
        if let Some(open) = *quote {
            if c == open {
                *quote = None;
            }
            return usize::from(open == '"' && c == '\\');
        }
        match c {
            '\\' => 1,
            '\'' | '"' => {
                *quote = Some(c);
                0
            }
            '>' => {
                self.redirects |= !matches!(rest, ['&', next, ..] if next.is_ascii_digit() || *next == '-');
                0
            }
            '<' => self.here(rest),
            _ => 0,
        }
    }

    fn here(&mut self, rest: &[char]) -> usize {
        match rest {
            ['<', '<', ..] => 2,
            ['<', ..] => {
                self.heredoc = true;
                1
            }
            _ => 0,
        }
    }
}
