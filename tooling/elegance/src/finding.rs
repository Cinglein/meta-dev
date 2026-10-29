use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding<Subject, Detail> {
    subject: Subject,
    detail: Detail,
}

impl<Subject, Detail> Finding<Subject, Detail> {
    #[must_use]
    pub fn new(subject: Subject, detail: Detail) -> Finding<Subject, Detail> {
        Finding { subject, detail }
    }

    #[must_use]
    pub fn detail(&self) -> &Detail {
        &self.detail
    }
}

impl<Subject: fmt::Display, Detail: fmt::Display> fmt::Display for Finding<Subject, Detail> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.subject, self.detail)
    }
}
