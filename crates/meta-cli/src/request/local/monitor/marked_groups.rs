mod mark;

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::group_sighting::GroupSighting;
use super::process_row::GroupId;
use crate::request::local::local_settings::MonitorSettings;
pub use mark::Mark;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MarkedGroups(BTreeMap<GroupId, Mark>);

impl MarkedGroups {
    pub fn holds(&self, group: GroupId) -> Option<()> {
        self.0.contains_key(&group).then_some(())
    }

    #[must_use]
    pub fn renewed(&self, sightings: &[GroupSighting], now: DateTime<Utc>, settings: &MonitorSettings) -> Self {
        let renewed = sightings.iter().filter_map(|sighting| {
            let before = self.0.get(&sighting.group()).copied();
            sighting.marked(before, now, settings).map(|mark| (sighting.group(), mark))
        });
        Self(renewed.collect())
    }
}
