mod laws;

pub use laws::Laws;

#[macro_export]
macro_rules! laws {
    ($($test:ident: $law:path => $source:expr;)+) => {
        $(
            #[test]
            fn $test() {
                use ::proptest::test_runner::{Config, TestRng, TestRunner};
                let config = Config {
                    cases: $crate::Laws::cases().unwrap().get(),
                    source_file: Some(file!()),
                    test_name: Some(stringify!($test)),
                    ..Config::default()
                };
                let rng = TestRng::deterministic_rng(config.rng_algorithm);
                let outcome = TestRunner::new_with_rng(config, rng).run(&$source, |value| {
                    ::proptest::prop_assert!($law(&value), "{:?}", value);
                    Ok(())
                });
                assert_eq!(outcome.map_err(|failure| failure.to_string()), Ok(()));
            }
        )+
    };
}
