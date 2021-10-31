
pub struct State {
    interactive: bool,
    
    // Each builtin changes (or resets) this;
    // Each invocation of an external command sets this.
    pub rv: i32,
}

impl State {
    pub fn new() -> Self {
        State { 
            interactive: true,
            rv: 0,
        }
    }

    pub fn is_interactive(&self) -> bool {
        self.interactive
    }
}
