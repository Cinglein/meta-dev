use meta_schema::{Placement, Rating, Target};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RatedItem {
    item: Target,
    placement: Placement,
    rating: Rating,
}

impl RatedItem {
    #[must_use]
    pub fn new(item: Target, placement: Placement, rating: Rating) -> RatedItem {
        RatedItem { item, placement, rating }
    }

    #[must_use]
    pub fn target(&self) -> Target {
        self.item
    }

    #[must_use]
    pub fn placement(&self) -> &Placement {
        &self.placement
    }

    #[must_use]
    pub fn rating(&self) -> &Rating {
        &self.rating
    }
}
