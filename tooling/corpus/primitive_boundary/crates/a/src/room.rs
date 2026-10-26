pub struct Room;
impl Room { pub fn label(&self, text: &str) -> bool { let kept = text.to_string(); text.trim().is_empty() } } //~ primitive-boundary: to_string handles //~ primitive-boundary: trim handles
impl Room { pub fn greet(&self) -> String { let fresh = String::new(); let parts = Vec::from(fresh); format!("{fresh}") } fn copy(&self) -> Label { String::from("room") } } //~ primitive-boundary: format handles //~ primitive-boundary: from handles
mod inner { impl Room { fn shout(text: &str) -> String { text.trim().to_owned() } } } //~ primitive-boundary: trim handles
#[cfg(test)] mod support;
