#[cfg(test)]
mod dev_only;
mod laws;
mod reading;

pub use laws::Laws;
pub use reading::Reading;

#[macro_export]
macro_rules! laws {
    ($($test:ident: $law:path => $source:expr;)+) => {
        $(
            #[test]
            fn $test() {
                use ::proptest::test_runner::{Config, TestRng, TestRunner};
                let config = Config {
                    cases: $crate::Laws::cases(::std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap().get(),
                    source_file: Some(file!()),
                    test_name: Some(stringify!($test)),
                    failure_persistence: None,
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
