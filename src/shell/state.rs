pub struct State {
    interactive: bool,

    // Each builtin and each external command sets (resets) this.
    // Shell constructs do not affect this.
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
