use std::fmt;
use std::process::ExitStatus;

use crate::request::output::Listed;

#[derive(Debug)]
pub struct Finished {
    status: ExitStatus,
    output: String,
    errors: String,
}

impl Finished {
    const SHOWN_REPLY_LEN: usize = 40;

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

impl fmt::Display for Finished {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shown: String = self.output.chars().take(Self::SHOWN_REPLY_LEN).collect();
        let reason = self
            .errors
            .lines()
            .next()
            .filter(|line| !line.is_empty())
            .map_or_else(|| shown.trim().chars().take(0).collect(), str::to_owned);
        formatter.write_str(&reason)
    }
}
