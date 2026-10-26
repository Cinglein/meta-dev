use serde_json::Value; use std::any::Any as Erased;
pub struct Settings { raw: Value, erased: Box<Erased>, table: toml::Value, local: Local } pub struct Local; //~ primitive-boundary: serde_json::Value in raw is a wire form //~ primitive-boundary: std::any::Any in erased //~ primitive-boundary: toml::Value in table
