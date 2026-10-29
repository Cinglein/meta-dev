#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Uncaught {
    Missed,
    TimedOut,
}

impl Uncaught {
    pub const ALL: [Uncaught; 2] = [Uncaught::Missed, Uncaught::TimedOut];

    pub fn file(self) -> &'static str {
        match self {
            Uncaught::Missed => "mutants.out/missed.txt",
            Uncaught::TimedOut => "mutants.out/timeout.txt",
        }
    }

    pub fn way_out(self) -> &'static str {
        match self {
            Uncaught::Missed => "missed: add a test that fails on each",
            Uncaught::TimedOut => {
                "timed out, never judged: make the tests that cover each faster; if they are fast, the --timeout guard is too tight"
            }
        }
    }
}
