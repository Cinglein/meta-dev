#[cfg(test)]
mod tests {
    use settings::EmbeddedDefaults;

    use crate::budget::Interval;
    use crate::policy::Policy;

    const OLD: &str = "total = 4000\n";
    const NEW: &str = "\n[budgets.test-time]\ndivision = \"performance-and-cost\"\nmeasure = \"test_wall_time\"\nallowance = { duration = 4000 }\nwindow = \"per_change\"\nsource = \"local_gate\"\n";

    #[test]
    fn a_limit_moved_into_a_budget_reads_as_the_old_key_did() {
        let moved =
            toml::from_str::<Policy>(&format!("{}{NEW}", Policy::DEFAULTS.replace(OLD, "")));
        assert_eq!(
            moved.map(|policy| policy.interval(Interval::Total)).ok(),
            Some(Policy::defaults().interval(Interval::Total))
        );
    }
}
