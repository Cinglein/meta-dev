use crate::task::adoption::gap::Gap;
use crate::task::adoption::required_tool::RequiredTool;
use crate::task::adoption::skip::Skip;

#[derive(Debug)]
pub struct StepRun {
    place: String,
    skip: Option<Skip>,
}

impl StepRun {
    pub fn new(place: String, skip: Option<Skip>) -> StepRun {
        StepRun { place, skip }
    }

    pub fn skip(&self) -> Option<&Skip> {
        self.skip.as_ref()
    }

    pub fn gap(&self, tool: RequiredTool) -> Gap {
        Gap::Skippable(tool, self.place.clone(), self.skip.clone().unwrap_or(Skip::Tolerated))
    }
}
