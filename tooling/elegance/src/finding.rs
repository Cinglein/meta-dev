use std::collections::BTreeSet;
use std::fmt;

use crate::located::Located;
use crate::places::Places;
use crate::source_path::SourcePath;

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

impl<Subject: Located, Detail: Located> Located for Finding<Subject, Detail> {
    fn places(&self, places: &Places) -> BTreeSet<SourcePath> {
        self.subject.places(places).into_iter().chain(self.detail.places(places)).collect()
    }
}

impl<Subject: fmt::Display, Detail: fmt::Display> fmt::Display for Finding<Subject, Detail> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.subject, self.detail)
    }
}
