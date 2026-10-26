pub struct Scale; impl Scale { pub fn run(&self) -> Settings { Weight::new(1).settings() } pub fn tally(&self) -> Plies { Tally::new(1).arm } }
