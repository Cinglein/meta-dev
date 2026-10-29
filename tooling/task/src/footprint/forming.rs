use super::Footprint;
use crate::policy::Policy;

pub struct Forming<'queue> {
    members: Vec<&'queue Footprint>,
    waiting: Vec<&'queue Footprint>,
}

impl<'queue> Forming<'queue> {
    pub fn batches(
        waiting: Vec<&'queue Footprint>,
        policy: &Policy,
        formed: Vec<Vec<&'queue Footprint>>,
    ) -> Vec<Vec<&'queue Footprint>> {
        if waiting.is_empty() {
            formed
        } else {
            let batch = waiting.into_iter().fold(
                Forming {
                    members: Vec::new(),
                    waiting: Vec::new(),
                },
                |forming, footprint| forming.offered(footprint, policy),
            );
            Self::batches(
                batch.waiting,
                policy,
                [formed, vec![batch.members]].concat(),
            )
        }
    }

    fn offered(self, footprint: &'queue Footprint, policy: &Policy) -> Forming<'queue> {
        if policy.queue().admits(&self.members)
            && self.members.iter().all(|member| !member.meets(footprint))
        {
            Forming {
                members: [self.members, vec![footprint]].concat(),
                waiting: self.waiting,
            }
        } else {
            Forming {
                members: self.members,
                waiting: [self.waiting, vec![footprint]].concat(),
            }
        }
    }
}
