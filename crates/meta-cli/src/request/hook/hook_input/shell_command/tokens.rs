mod scanner;

use scanner::Scanner;

#[derive(Debug, Default)]
pub struct Tokens {
    segments: Vec<Vec<String>>,
    redirects: bool,
    heredoc: bool,
}

impl Tokens {
    const HEREDOC: &'static str = "shell rule: no heredocs; write the file with the Write tool";
    const REDIRECT: &'static str = "shell rule: no output redirection to a file, /dev/null included; write with the Write or Edit tool; pipes and reading are fine";

    pub fn scan(text: &str) -> Self {
        Scanner::tokens(text)
    }

    pub fn segments(&self) -> impl Iterator<Item = &[String]> {
        self.segments.iter().map(Vec::as_slice)
    }

    pub fn breach(&self) -> Option<&'static str> {
        self.heredoc.then_some(Self::HEREDOC).or(self.redirects.then_some(Self::REDIRECT))
    }
}
