use std::env;

use task::Failure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunEvent {
    Pull(Option<String>),
    Other(String),
}

impl RunEvent {
    const EVENT: &str = "GITHUB_EVENT_NAME";
    const PULL_EVENT: &str = "pull_request";
    const NUMBER: &str = "PULL_REQUEST";

    pub fn ambient() -> RunEvent {
        RunEvent::seen(env::var(Self::EVENT).ok(), env::var(Self::NUMBER).ok())
    }

    pub fn seen(event: Option<String>, number: Option<String>) -> RunEvent {
        event
            .filter(|event| !event.is_empty() && !event.starts_with(Self::PULL_EVENT))
            .map_or_else(|| RunEvent::Pull(number.filter(|number| !number.is_empty())), RunEvent::Other)
    }

    pub fn number(self) -> Result<Option<String>, Failure> {
        match self {
            RunEvent::Pull(number) => number.map(Some).ok_or(Failure::Unset(Self::NUMBER)),
            RunEvent::Other(event) => {
                println!("stacked: a {event} run has no pull request to check");
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RunEvent;

    const EVENTS: [(&str, &str); 3] = [("push", "7"), ("pull_request_target", ""), ("", "7")];

    #[test]
    fn a_run_outside_a_pull_request_has_nothing_to_check_and_a_pull_request_needs_its_number() {
        let seen = EVENTS.map(|(event, number)| RunEvent::seen(Some(event.into()), Some(number.into())).number().ok());
        assert_eq!(seen, [Some(None), None, Some(Some(EVENTS[2].1.into()))]);
    }
}
