mod change_link;
mod exposure;
mod open_item;

use meta_schema::{Division, NewTask};
use meta_severity::{Scales, SeverityFilter};
use meta_state::{State, TaskState};

use super::budget::{Amount, ItemFilter, ItemKind};
use crate::policy::Most;
pub use change_link::ChangeLink;
pub use exposure::Exposure;
use open_item::OpenItem;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Standing {
    items: Vec<OpenItem>,
    linked: Vec<NewTask>,
    scales: Scales,
}

impl Standing {
    #[must_use]
    pub fn read(state: &State, link: &ChangeLink, scales: &Scales) -> Standing {
        let open = |entry| state.closed(entry).is_none();
        let concerns = state
            .concerns()
            .iter()
            .filter(|concern| open(concern.target().entry()))
            .map(|concern| {
                OpenItem::new(
                    Some(concern.target()),
                    concern.new_concern().placement().clone(),
                    vec![ItemFilter::Kind(ItemKind::Concern)],
                    state.rated_item(concern.target()),
                )
            });
        let issues = state
            .issues()
            .iter()
            .filter(|issue| open(issue.target().entry()))
            .map(|issue| {
                OpenItem::new(
                    Some(issue.target()),
                    issue.new_issue().placement().clone(),
                    vec![ItemFilter::Kind(ItemKind::Issue)],
                    state.rated_item(issue.target()),
                )
            });
        let tasks = state
            .tasks()
            .iter()
            .filter(|task| open(task.entry()))
            .map(|task| Self::task(state, task));
        Standing {
            items: concerns.chain(issues).chain(tasks).collect(),
            linked: state
                .tasks()
                .iter()
                .filter(|task| link.links(task) && state.retirement(task.entry()).is_none())
                .map(|task| task.new_task().clone())
                .collect(),
            scales: scales.clone(),
        }
    }

    #[must_use]
    pub fn spend(&self, filter: &ItemFilter, division: &Division) -> Amount {
        Amount::Count(Most::from(self.counted(filter, division).count()))
    }

    #[must_use]
    pub fn least_severe(&self, division: &Division) -> Option<ItemFilter> {
        self.scales
            .scale(division)
            .and_then(|scale| scale.levels().last())
            .map(|level| ItemFilter::Severity(SeverityFilter::AtLevel(level.name().clone())))
    }

    #[must_use]
    pub fn exposure(&self, filter: Option<&ItemFilter>, division: &Division) -> Exposure {
        if filter.is_some_and(|filter| self.paid_down(filter, division)) {
            Exposure::PaysDown
        } else if self.within(division) {
            Exposure::Within
        } else {
            Exposure::Outside
        }
    }

    fn task(state: &State, task: &TaskState) -> OpenItem {
        let marks = [
            Some(ItemFilter::Kind(ItemKind::Task)),
            state
                .status_of(task)
                .discussion()
                .map(|_| ItemFilter::InDiscussion),
            task.progress()
                .assignments()
                .is_empty()
                .then_some(ItemFilter::UnheldTasks),
        ];
        OpenItem::new(
            None,
            task.new_task().placement().clone(),
            marks.into_iter().flatten().collect(),
            None,
        )
    }

    fn paid_down(&self, filter: &ItemFilter, division: &Division) -> bool {
        !self.linked.is_empty()
            && self.linked.iter().all(|task| {
                self.items
                    .iter()
                    .filter_map(|item| item.pays_into(filter, division, &self.scales))
                    .any(|item| item.addressed_by(task))
            })
    }

    fn within(&self, division: &Division) -> bool {
        self.linked.is_empty()
            || self.linked.iter().any(|task| {
                task.placement()
                    .division()
                    .is_none_or(|linked| division == linked || division.strictly_contains(linked))
            })
    }

    fn counted<'scan>(
        &'scan self,
        filter: &'scan ItemFilter,
        division: &'scan Division,
    ) -> impl Iterator<Item = &'scan OpenItem> {
        self.items
            .iter()
            .filter(move |item| item.counts_in(filter, division, &self.scales))
    }
}

#[cfg(test)]
mod tests;
