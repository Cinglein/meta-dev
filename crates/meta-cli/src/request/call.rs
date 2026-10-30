use std::io::Read;

use super::Request;
use super::answer::Answer;
use super::environment::Environment;
use super::local::TraceRecord;

pub struct Call<'input> {
    words: Vec<String>,
    environment: Environment,
    input: Box<dyn Read + 'input>,
}

impl<'input> Call<'input> {
    #[must_use]
    pub fn new(words: Vec<String>, environment: Environment, input: Box<dyn Read + 'input>) -> Self {
        Self { words, environment, input }
    }

    #[must_use]
    pub fn answered(self) -> Answer {
        let trace = TraceRecord::started(&self.words, &self.environment);
        let request = Request::try_from(self.words);
        let answered = request.and_then(|request| request.answer(&self.environment, self.input));
        trace.kept(&self.environment);
        Answer::new(answered, self.environment)
    }
}

impl std::fmt::Debug for Call<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Call").field("words", &self.words).field("environment", &self.environment).finish()
    }
}
