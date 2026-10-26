use task::Violation;

pub enum Sighting {
    Number(Violation),
    Size(Violation),
    Nothing,
}

impl Sighting {
    pub fn filed(self, numbers: &mut Vec<Violation>, sizes: &mut Vec<Violation>) {
        match self {
            Sighting::Number(violation) => numbers.push(violation),
            Sighting::Size(violation) => sizes.push(violation),
            Sighting::Nothing => {}
        }
    }
}
