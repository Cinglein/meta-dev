use super::Policy;
use super::vocabulary::Vocabulary;

pub struct Vague;

impl Vocabulary for Vague {
    fn entries(policy: &Policy) -> &[String] {
        &policy.vague
    }
}
