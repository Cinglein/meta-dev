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
}

impl ShellRules {
    pub fn forbidden_in<'a>(&self, segment: &'a [String]) -> Option<&'a str> {
        let mut wrapped = false;
        for word in segment.iter().map(String::as_str) {
            let skipped = word.contains('=')
                || (wrapped && (word.starts_with('-') || word.starts_with(|c: char| c.is_ascii_digit())));
            if self.wrappers.iter().any(|wrapper| wrapper.names(word)) {
                wrapped = true;
            } else if !skipped {
                return self.forbidden.iter().any(|program| program.names(word)).then_some(word);
            }
        }
        None
    }
}
