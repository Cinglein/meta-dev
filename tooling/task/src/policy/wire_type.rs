use super::Policy;
use super::vocabulary::Vocabulary;

#[derive(Debug)]
pub struct WireType;

impl Vocabulary for WireType {
    fn entries(policy: &Policy) -> &[String] {
        &policy.wire_types
    }
}
