use meta_run::{EnvPolicy, Run};
use serde::Deserialize;

use crate::places::Places;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Setup {
    run: Vec<String>,
}

impl Setup {
    pub fn run(&self, places: &Places) -> Run {
        let words: Vec<_> = self.run.iter().map(|word| places.shown(word)).collect();
        let run = |(program, operands): (_, &[_])| Run::new(program, EnvPolicy::NoGitVars).args(operands);
        words.split_first().map_or_else(|| Run::new("", EnvPolicy::NoGitVars), run).dir(places.root())
    }
}
