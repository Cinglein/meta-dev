use crate::env_policy::EnvPolicy;

pub trait Tool: Copy {
    const PROGRAM: &'static str;
    const POLICY: EnvPolicy;

    fn words(self) -> &'static [&'static str];
}
