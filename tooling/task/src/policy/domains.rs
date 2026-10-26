#[cfg(test)]
mod tests {
    use crate::policy::Policy;
    use settings::EmbeddedDefaults;

    const NESTED: &str = "\n[domains.\"performance-and-cost/unit\"]\ntests = [\"meta_*\"]\n\n[budgets.unit-time]\ndivision = \"performance-and-cost\"\nmeasure = \"test_wall_time\"\nallowance = { duration = 4000 }\nwindow = \"per_change\"\nsource = \"local_gate\"\nwarning = 50\ntolerance = 10\n";
    const REFUSED: [(&str, &str); 6] = [
        (
            "\n[domains.\"performance-and-cost/unit\"]\ntests = [\"*\"]\nworkflows = [\"*\"]\n",
            "domain performance-and-cost/unit: not strictly inside its parent's domain",
        ),
        (
            "\n[domains.\"performance-and-cost/unit\"]\ncrates = [\"meta-cli\"]\n",
            "domain performance-and-cost/unit: not strictly inside its parent's domain",
        ),
        (
            "\n[domains.architecture-and-planning]\npaths = [\"context*\"]\n",
            "domain architecture-and-planning: planning has no code scopes",
        ),
        (
            "\n[budgets.lines]\ndivision = \"features-and-maintenance\"\nmeasure = \"test_wall_time\"\nallowance = { duration = 4000 }\nwindow = \"per_change\"\nsource = \"ci\"\n",
            "budget lines: its measure needs tests, which its division's domain does not declare",
        ),
        (
            "\n[budgets.time]\ndivision = \"performance-and-cost\"\nmeasure = \"test_wall_time\"\nallowance = { count = 4 }\nwindow = \"per_change\"\nsource = \"ci\"\n",
            "budget time: its allowance is not in its measure's unit",
        ),
        (
            "\n[budgets.time]\ndivision = \"performance-and-cost\"\nmeasure = \"test_wall_time\"\nallowance = { duration = 4000 }\nwindow = \"per_change\"\nsource = \"ci\"\nwarning = 100\n",
            "warning line 100: below the blocking line at 100",
        ),
    ];

    fn read(added: &str) -> Result<Policy, String> {
        toml::from_str::<Policy>(&format!("{}{added}", Policy::DEFAULTS))
            .map_err(|error| error.message().to_owned())
    }

    #[test]
    fn a_nested_domain_inside_its_parent_and_a_budget_on_a_declared_scope_are_read() {
        assert_eq!(read(NESTED).err(), None);
    }

    #[test]
    fn domains_outside_their_parent_planning_code_and_unmeasurable_budgets_are_refused() {
        let refusals = REFUSED.map(|(added, _)| read(added).err());
        assert_eq!(
            refusals,
            REFUSED.map(|(_, refusal)| Some(refusal.to_owned()))
        );
    }
}
