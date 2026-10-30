mod program;

use program::Program;
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShellRules {
    #[serde(default)]
    forbidden: Vec<Program>,
    #[serde(default)]
    wrappers: Vec<Program>,
    #[serde(default)]
    valued: Vec<Program>,
    #[serde(default)]
    interpreters: Vec<Program>,
}

impl ShellRules {
    const QUERIED: &'static str = "command";
    const QUERIES: &'static [&'static str] = &["-v", "-V"];
    const EVAL: &'static str = "eval";
    const COMMAND_FLAG: char = 'c';

    pub fn forbidden_in<'a>(&self, segment: &'a [String]) -> Option<&'a str> {
        let word = segment.get(self.command_at(segment)?)?.as_str();
        self.forbidden.iter().any(|program| program.names(word)).then_some(word)
    }

    pub fn nested(&self, segment: &[String]) -> Option<String> {
        let (word, args) = segment.get(self.command_at(segment)?..)?.split_first()?;
        self.interpreters.iter().find(|program| program.names(word))?;
        let flag = args
            .iter()
            .position(|arg| arg.starts_with('-') && !arg.starts_with("--") && arg.contains(Self::COMMAND_FLAG));
        match flag {
            Some(at) => args.get(at + 1).cloned(),
            None => (word == Self::EVAL).then(|| args.join(" ")),
        }
    }

    fn command_at(&self, segment: &[String]) -> Option<usize> {
        let mut words = segment.iter().enumerate();
        let mut wrapped = false;
        while let Some((index, word)) = words.next() {
            let option = wrapped && (word.starts_with('-') || word.starts_with(|c: char| c.is_ascii_digit()));
            if self.wrappers.iter().any(|wrapper| wrapper.names(word)) {
                let query = segment.get(index + 1).is_some_and(|next| Self::QUERIES.contains(&next.as_str()));
                if word == Self::QUERIED && query {
                    return None;
                }
                wrapped = true;
            } else if wrapped && self.valued.iter().any(|flag| flag.names(word)) {
                words.next();
            } else if !(option || word.contains('=')) {
                return Some(index);
            }
        }
        None
    }
}
