pub struct SideBySide;

impl SideBySide {
    pub fn run(scenarios: &[fn()]) {
        let failures: Vec<_> = std::thread::scope(|scope| {
            scenarios
                .iter()
                .map(|scenario| scope.spawn(scenario))
                .collect::<Vec<_>>()
                .into_iter()
                .filter_map(|started| started.join().err())
                .collect()
        });
        assert!(failures.is_empty(), "{} scenarios failed", failures.len());
    }
}
