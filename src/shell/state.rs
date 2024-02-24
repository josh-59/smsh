pub struct State {
    interactive: bool,

    // True if shell is to carry out parsing only
    no_exec: bool,

    // Each builtin and each external command sets (resets) this.
    // Shell constructs do not affect this.
    pub rv: i32,
}

impl State {
    pub fn new() -> Self {
        State {
            interactive: true,
            no_exec: false,
            rv: 0,
        }
    }

    pub fn is_interactive(&self) -> bool {
        self.interactive
    }

    pub fn no_exec(&self) -> bool {
        self.no_exec
    }
}
