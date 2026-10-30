use meta_acceptance::{AcceptanceRow, Enforcement, Gating, Observability, SpanName};
use meta_schema::{Acceptance, Id, Project, Root};

use super::missing_option::MissingOption;
use crate::request::cli_failure::{CliFailure, Invalid, Misuse};

#[derive(Clone, Debug, Default)]
pub struct AcceptanceOptions {
    root: Option<Root>,
    rows: Vec<AcceptanceRow>,
    spans: Vec<SpanName>,
    gates: Vec<Enforcement>,
}

impl AcceptanceOptions {
    #[must_use]
    pub fn rooted(self, root: Root) -> Self {
        Self { root: Some(root), ..self }
    }

    #[must_use]
    pub fn row(self, row: AcceptanceRow) -> Self {
        Self { rows: [self.rows, vec![row]].concat(), ..self }
    }

    #[must_use]
    pub fn span(self, span: SpanName) -> Self {
        Self { spans: [self.spans, vec![span]].concat(), ..self }
    }

    #[must_use]
    pub fn gate(self, gate: Enforcement) -> Self {
        Self { gates: [self.gates, vec![gate]].concat(), ..self }
    }

    pub fn accepting(&self, project: Id<Project>) -> Result<Acceptance, CliFailure> {
        let missing = CliFailure::raised::<Misuse>;
        let root = self.root.ok_or_else(|| missing(MissingOption::Root))?;
        let rows = Some(self.rows.clone()).filter(|rows| !rows.is_empty());
        let rows = rows.ok_or_else(|| missing(MissingOption::Row))?;
        let spans = Some(self.spans.clone()).filter(|spans| !spans.is_empty());
        let spans = spans.ok_or_else(|| missing(MissingOption::Span))?;
        let gates = Some(self.gates.clone()).filter(|gates| !gates.is_empty());
        let gates = gates.ok_or_else(|| missing(MissingOption::Gate))?;
        let observability = Observability::named(rows, spans).map_err(CliFailure::raised::<Invalid>)?;
        let gating = Gating::try_from(gates).map_err(CliFailure::raised::<Invalid>)?;
        Ok(Acceptance::new(project, root, observability, gating))
    }
}
