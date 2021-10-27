pub struct State {
    interactive: bool
}

impl State {
    pub fn new() -> Self {
        State { 
            interactive: true
        }
    }

    pub fn is_interactive(&self) -> bool {
        self.interactive
    }
}
