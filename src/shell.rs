use anyhow::Result;
use super::source::{Source, TTY};
use super::line::Line;
use super::builtins::{Builtin, chdir};

use std::collections::HashMap;

pub struct Shell {
    sources: Vec<Box<dyn Source>>,
    builtins: HashMap<&'static str, Builtin>,
}

impl Shell {
    pub fn new() -> Result<Shell> {
        let tty = TTY::new()?;
        let sources = vec![tty];

        let mut builtins = HashMap::<&'static str, Builtin>::new();
        builtins.insert("cd", chdir);

        Ok(Shell{ sources, builtins })
    }

    pub fn run(&mut self) -> Result<()> {

        while let Some(mut line) = self.get_line()? {
            line.execute()?;
        }

        Ok(())
    }

    fn get_line(&mut self) -> Result<Option<Line>> {
        if let Some(mut source) = self.sources.pop() {
            if let Some(line) = source.get_line(">> ")? {
                self.sources.push(source);
                Ok(Some(line))
            } else {
                self.get_line()
            }
        } else {
            Ok(None)
        }
    }
}
