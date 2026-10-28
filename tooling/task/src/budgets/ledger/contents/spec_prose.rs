use std::convert::Infallible;
use std::str::FromStr;

use crate::policy::{Most, SpecSettings};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpecProse(Most);

impl SpecProse {
    #[must_use]
    pub fn most(self) -> Most {
        self.0
    }
}

impl FromStr for SpecProse {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<SpecProse, Infallible> {
        let (rendered, unconverted) =
            text.split_once(SpecSettings::UNCONVERTED)
                .map_or((text, ""), |(rendered, marked)| {
                    (
                        rendered,
                        marked.split_once('\n').map_or("", |(_, prose)| prose),
                    )
                });
        let prose = rendered
            .lines()
            .filter(|line| !line.starts_with(SpecSettings::RULE))
            .chain(unconverted.lines());
        Ok(SpecProse(Most::from(
            prose.flat_map(str::split_whitespace).count(),
        )))
    }
}
