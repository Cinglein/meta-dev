use std::collections::BTreeSet;
use std::fmt;

use crate::listed::Listed;
use crate::smell_inventory::{SmellId, SmellInventory};
use crate::source_path::SourcePath;
use crate::tally::Tally;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recall {
    tag: &'static str,
    tagged: BTreeSet<SmellId>,
    flagged: BTreeSet<SmellId>,
    noise: Tally,
    findings: Tally,
}

impl Recall {
    #[must_use]
    pub fn scored(tag: &'static str, findings: &[BTreeSet<SourcePath>], inventory: &SmellInventory) -> Recall {
        let flags = |spots| inventory.flagged(spots).into_iter().cloned().collect::<BTreeSet<SmellId>>();
        Recall {
            tag,
            tagged: inventory.tagged(tag).into_iter().cloned().collect(),
            flagged: findings.iter().flat_map(flags).collect(),
            noise: Tally::counting(findings.iter().filter(|spots| inventory.flagged(spots).is_empty())),
            findings: Tally::counting(findings),
        }
    }
}

impl fmt::Display for Recall {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hit = Listed::new(self.tagged.intersection(&self.flagged));
        let other = Listed::new(self.flagged.difference(&self.tagged));
        write!(
            formatter,
            "recall {} of {} tagged {} [{hit}]; other entries flagged {} [{other}]; noise {} of {} findings",
            Tally::counting(self.tagged.intersection(&self.flagged)),
            self.tagged.len(),
            self.tag,
            Tally::counting(self.flagged.difference(&self.tagged)),
            self.noise,
            self.findings
        )
    }
}
