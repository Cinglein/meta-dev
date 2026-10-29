mod alert_argument;
mod open_alerts;
mod runner;

use task::{Program, Stdout, Workspace};

use crate::task::failure::Failure;
use alert_argument::AlertArgument;
use open_alerts::OpenAlerts;
use runner::Runner;

#[derive(Debug)]
pub struct Supply;

impl Supply {
    const OPEN_ALERTS: &str = "repos/{owner}/{repo}/dependabot/alerts?state=open&per_page=100";
    const ALERT_LINE: &str = r#".[] | "\(.security_advisory.severity) \(.dependency.package.ecosystem) \(.dependency.package.name) \(.security_advisory.ghsa_id) \(.html_url)""#;

    pub fn alerts(workspace: &Workspace) -> Result<(), Failure> {
        workspace
            .line(Program::GitHub)
            .argument(AlertArgument::Api)
            .argument(AlertArgument::Paginate)
            .argument(Self::OPEN_ALERTS)
            .argument(AlertArgument::Jq)
            .argument(Self::ALERT_LINE)
            .captured::<Stdout>()
            .map(|listed| {
                let Ok(open) = listed.parse::<OpenAlerts>();
                println!("{open}");
            })
            .or_else(|failure| Runner::ambient().unreached(Failure::from(failure), OpenAlerts::NAME))
    }
}
