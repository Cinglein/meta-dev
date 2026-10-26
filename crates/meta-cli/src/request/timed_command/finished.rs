use std::process::ExitStatus;

use crate::request::output::Listed;

#[derive(Debug)]
pub struct Finished {
    status: ExitStatus,
    output: String,
    errors: String,
}

impl Finished {
    pub fn new(status: ExitStatus, output: String, errors: String) -> Self {
        Self { status, output, errors }
    }

    pub fn succeeded(self) -> Result<String, Listed> {
        let complaint = self.complaint();
        self.status.success().then_some(self.output).ok_or(complaint)
    }

    pub fn output(&self) -> &str {
        &self.output
    }

    pub fn complaint(&self) -> Listed {
        Listed::new(self.errors.lines().next().unwrap_or_default())
    }
}
