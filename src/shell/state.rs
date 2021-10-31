
pub struct State {
    interactive: bool,
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
