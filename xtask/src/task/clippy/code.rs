use serde::Deserialize;

use super::code_name::CodeName;

#[derive(Debug, Deserialize)]
pub(super) struct Code {
    code: CodeName,
}

impl Code {
    pub(super) fn is_lint(&self) -> bool {
        self.code.is_lint()
    }
}
