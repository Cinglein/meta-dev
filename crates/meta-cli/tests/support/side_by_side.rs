pub struct SideBySide;

impl SideBySide {
    pub fn run(scenarios: &[(&'static str, fn())]) {
        let failures: Vec<_> = std::thread::scope(|scope| {
            scenarios
                .iter()
                .map(|(name, scenario)| {
                    std::thread::Builder::new().name((*name).to_owned()).spawn_scoped(scope, scenario)
                })
                .collect::<Vec<_>>()
                .into_iter()
                .filter_map(|started| started.unwrap().join().err())
                .collect()
        });
        assert!(failures.is_empty(), "{} scenarios failed", failures.len());
    }
}
