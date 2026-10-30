use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct Program(String);

impl Program {
    const QUERIES: &'static [&'static str] = &["-v", "-V"];
    const COMMAND_FLAG: char = 'c';
    const EVAL: &'static str = "eval";

    pub fn script(word: &str, args: &[String]) -> Option<String> {
        Self::payload(args).or_else(|| (word == Self::EVAL).then(|| args.join(" ")))
    }

    fn payload(args: &[String]) -> Option<String> {
        let flagged =
            |arg: &&String| arg.starts_with('-') && !arg.starts_with("--") && arg.contains(Self::COMMAND_FLAG);
        args.iter().skip_while(|arg| !flagged(arg)).nth(1).cloned()
    }

    pub fn queries(next: Option<&String>) -> bool {
        next.is_some_and(|word| Self::QUERIES.contains(&word.as_str()))
    }

    pub fn option(word: &str) -> bool {
        word.starts_with('-') || word.starts_with(|c: char| c.is_ascii_digit())
    }

    pub fn base(word: &str) -> &str {
        word.rsplit('/').next().unwrap_or(word)
    }

    pub fn names(&self, word: &str) -> bool {
        let base = Self::base(word);
        base == self.0 || base.trim_end_matches(|c: char| c.is_ascii_digit() || c == '.') == self.0
    }
}
