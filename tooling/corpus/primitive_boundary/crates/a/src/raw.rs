use serde_json::Value; pub struct Raw(Value); impl Raw { fn inner(&self) -> &Value { &self.0 } }
