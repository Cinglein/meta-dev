use std::fmt;
use std::process::ExitCode;

use meta_run::{EnvPolicy, Run};

use super::call::Call;
use super::cli_failure::CliFailure;
use super::environment::Environment;
use super::follow_up::FollowUp;
use super::output::Output;

#[derive(Debug)]
pub struct Answer {
    answered: Result<Output, CliFailure>,
    then: Vec<FollowUp>,
    environment: Environment,
}

impl Answer {
    pub(crate) fn new(answered: Result<Output, CliFailure>, environment: Environment) -> Self {
        Self { answered, then: environment.follow_ups(), environment }
    }

    #[must_use]
    pub fn err(&self) -> Option<String> {
        self.answered.as_ref().err().map(|failure| format!("meta: {failure}\n"))
    }

    #[must_use]
    pub fn code(&self) -> ExitCode {
        self.answered.as_ref().map_or(ExitCode::FAILURE, Output::exit_code)
    }

    #[must_use]
    pub fn then(&self) -> Vec<Call<'static>> {
        let called = |follow_up: &FollowUp| {
            let environment = self.environment.moved_to(follow_up.directory().to_path_buf());
            Call::new(follow_up.words().to_vec(), environment, Box::new(std::io::empty()))
        };
        self.then.iter().map(called).collect()
    }

    #[must_use]
    pub fn delivered(self) -> ExitCode {
        print!("{self}");
        self.err().into_iter().for_each(|err| eprint!("{err}"));
        self.then.iter().for_each(|follow_up| {
            let started = Run::new(self.environment.program(), EnvPolicy::Inherit)
                .args(follow_up.words())
                .dir(follow_up.directory())
                .detached();
            started.err().into_iter().for_each(|failure| eprintln!("meta: follow-up not started: {failure}"));
        });
        self.code()
    }
}

impl fmt::Display for Answer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.answered.as_ref().map_or(Ok(()), |output| write!(formatter, "{output}"))
    }
}
