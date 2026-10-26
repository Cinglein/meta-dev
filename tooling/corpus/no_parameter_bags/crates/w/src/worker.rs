pub struct Worker; impl Worker { pub fn run(&self) -> Plies { Bag::new(1).arm() + Record::new(1).arm() + Crate::new(1).arm } pub fn probe() -> Probe { Probe } }
