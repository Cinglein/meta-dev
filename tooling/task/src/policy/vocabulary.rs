use super::Policy;

pub trait Vocabulary {
    fn entries(policy: &Policy) -> &[String];
}
