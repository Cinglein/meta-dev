use super::super::culprit::Culprit;
use super::super::judged_row::JudgedRow;
use super::super::ownership::Ownership;
use super::super::process_row::ProcessRow;
use crate::request::local::local_settings::MonitorSettings;

#[derive(Debug, Default)]
pub struct JudgedRows(Vec<JudgedRow>);

impl JudgedRows {
    pub fn new(rows: Vec<JudgedRow>) -> Self {
        Self(rows)
    }

    pub fn culprits(&self, wanted: Ownership, settings: &MonitorSettings) -> Vec<Culprit> {
        let owned = self.0.iter().filter_map(|row| row.owned(wanted));
        owned.filter_map(|row| row.cause(settings).map(|cause| row.culprit(cause, self.whole(row)))).collect()
    }

    pub fn survivors(&self, culprits: &[Culprit]) -> Vec<Culprit> {
        let alive = self.0.iter().filter_map(|row| row.owned(Ownership::Agent));
        alive
            .filter_map(|row| {
                let culprit = culprits.iter().find(|culprit| culprit.names(row.pid(), row.command()).is_some())?;
                Some(row.culprit(culprit.cause(), self.whole(row)))
            })
            .collect()
    }

    pub fn tracked(&self) -> impl Iterator<Item = &ProcessRow> {
        self.0.iter().filter_map(JudgedRow::tracked)
    }

    pub fn shown(&self) -> Vec<JudgedRow> {
        self.0.iter().filter_map(JudgedRow::shown).cloned().collect()
    }

    fn whole(&self, row: &ProcessRow) -> Option<()> {
        self.0
            .iter()
            .filter(|other| other.row().group() == row.group())
            .all(|member| member.owned(Ownership::Agent).is_some())
            .then_some(())
    }
}
