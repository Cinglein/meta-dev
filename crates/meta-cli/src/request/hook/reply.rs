mod blocking;
mod denial;
mod verdict;

use std::fmt;

use serde_json::{Map, Value, json};

use super::HookEvent;
use crate::request::output::Listed;
pub use blocking::Blocking;
pub use denial::Denial;
pub use verdict::Verdict;

#[derive(Debug)]
pub struct Reply {
    event: HookEvent,
    verdict: Verdict,
    context: Vec<Listed>,
}

impl Reply {
    pub fn new(event: HookEvent, verdict: Verdict, context: Option<Listed>) -> Self {
        Self { event, verdict, context: context.into_iter().collect() }
    }

    pub fn denied(&self) -> bool {
        self.verdict.denial().is_some()
    }

    pub fn with_news(mut self, news: Option<Listed>) -> Self {
        self.context.extend(news);
        self
    }
}

impl fmt::Display for Reply {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let context: Vec<String> = self.context.iter().map(|text| format!("meta-development: {text}")).collect();
        let shown = Some(context.join("\n")).filter(|joined| !joined.is_empty());
        let specific: Map<String, Value> = [
            ("hookEventName", Some(json!(self.event.to_string()))),
            ("additionalContext", shown.map(Value::from)),
            ("permissionDecision", self.verdict.denial().map(|_| json!("deny"))),
            ("permissionDecisionReason", self.verdict.denial().map(|reason| json!(reason.to_string()))),
        ]
        .into_iter()
        .filter_map(|(key, value)| value.map(|value| (key.to_owned(), value)))
        .collect();
        let top: Map<String, Value> = [
            ("hookSpecificOutput", Some(Value::Object(specific))),
            ("decision", self.verdict.blocking().map(|_| json!("block"))),
            ("reason", self.verdict.blocking().map(|reason| json!(reason.to_string()))),
        ]
        .into_iter()
        .filter_map(|(key, value)| value.map(|value| (key.to_owned(), value)))
        .collect();
        match (self.verdict.allowing(), self.context.is_empty()) {
            (Some(_), true) => Ok(()),
            _ => write!(formatter, "{}", Value::Object(top)),
        }
    }
}
