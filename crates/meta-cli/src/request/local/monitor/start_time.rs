#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartTime(String);

impl StartTime {
    pub fn read(text: &str) -> Option<Self> {
        let text = text.trim();
        (!text.is_empty()).then(|| Self(text.to_owned()))
    }
}
