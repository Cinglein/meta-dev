use std::fmt;

use meta_run::Ran;

#[derive(Debug)]
pub struct Unanswered<'ran>(&'ran Ran);

impl<'ran> Unanswered<'ran> {
    const SHOWN_REPLY_LEN: usize = 40;

    pub fn new(ran: &'ran Ran) -> Self {
        Self(ran)
    }
}

impl fmt::Display for Unanswered<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(complaint) = self.0.stderr().first_line() {
            formatter.write_str(complaint)
        } else {
            let shown: String = self.0.stdout().text().chars().take(Self::SHOWN_REPLY_LEN).collect();
            write!(formatter, "reply not JSON: {}", shown.trim())
        }
    }
}
