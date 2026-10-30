#[derive(Clone, Copy, Debug)]
pub enum Breach {
    Heredoc,
    HereString,
    Redirect,
}

impl Breach {
    const HEREDOC: &'static str = "shell rule: no heredocs; write the file with the Write tool";
    const HERE_STRING: &'static str =
        "shell rule: no here-strings; pass the text as an argument or write it with the Write tool";
    const REDIRECT: &'static str = "shell rule: no output redirection to a file, /dev/null included; write with the Write or Edit tool; pipes and reading are fine";
    const DELIMITER_STARTS: &'static [char] = &['_', '\'', '"', '\\'];

    pub fn reason(self) -> &'static str {
        match self {
            Self::Heredoc => Self::HEREDOC,
            Self::HereString => Self::HERE_STRING,
            Self::Redirect => Self::REDIRECT,
        }
    }

    pub fn heredoc(tail: &[char]) -> Option<Self> {
        let first = tail.iter().skip_while(|c| **c == '-').find(|c| !c.is_whitespace());
        first.is_some_and(|c| c.is_alphabetic() || Self::DELIMITER_STARTS.contains(c)).then_some(Self::Heredoc)
    }
}
