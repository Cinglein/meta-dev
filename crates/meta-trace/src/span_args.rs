use std::fmt;

use serde::{Deserialize, Serialize};
use tracing::field::{Field, Visit};

use crate::field_text::FieldText;
use crate::span_field::SpanField;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpanArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    label: Option<FieldText>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    program: Option<FieldText>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    args: Option<FieldText>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    network: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    exit: Option<FieldText>,
}

impl SpanArgs {
    pub fn program(&self) -> Option<&FieldText> {
        self.program.as_ref()
    }

    pub fn networked(&self) -> bool {
        self.network == Some(true)
    }
}

impl Visit for SpanArgs {
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.network = (field.name().parse() == Ok(SpanField::Network)).then_some(value).or(self.network);
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        let shown = Some(FieldText::shown(value));
        match field.name().parse() {
            Ok(SpanField::Label) => self.label = shown,
            Ok(SpanField::Program) => self.program = shown,
            Ok(SpanField::Args) => self.args = shown,
            Ok(SpanField::Exit) => self.exit = shown,
            Ok(SpanField::Network) | Err(()) => {}
        }
    }
}

impl fmt::Display for SpanArgs {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let words = [&self.label, &self.program, &self.args]
            .into_iter()
            .flatten()
            .filter(|word| *word != &FieldText::default());
        let shown: Vec<String> = words.map(ToString::to_string).collect();
        formatter.write_str(&shown.join(" "))
    }
}
