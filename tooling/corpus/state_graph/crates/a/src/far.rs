pub enum Wire { Far(Volt) } impl Wire { fn own(&self) -> Volt { match self { Wire::Far(n) => *n } } }
