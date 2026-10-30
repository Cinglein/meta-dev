use super::option_name::OptionName;
use super::word::Word;

#[derive(Clone, Debug)]
pub struct Given {
    name: OptionName,
    value: Word,
}

impl Given {
    pub fn new(name: OptionName, value: Word) -> Self {
        Self { name, value }
    }

    pub fn value_of(&self, name: OptionName) -> Option<&Word> {
        (self.name == name).then_some(&self.value)
    }
}
