use anyhow::Result;
use super::source::{Source, TTY};
use super::line::Line;

use std::collections::HashMap;

mod modules;
use modules::*;

pub struct Shell {
    sources: Vec<Box<dyn Source>>,
    builtins: HashMap<&'static str, Builtin>,
}

impl Shell {
    pub fn new() -> Result<Shell> {
        let tty = TTY::new()?;
        let sources = vec![tty];

        let builtins = HashMap::<&'static str, Builtin>::new();

        let mut smsh = Shell { sources, builtins };

        load_module(&mut smsh, Module::Core)?;

        Ok(smsh)
    }

    pub fn run(&mut self) -> Result<()> {

        while let Some(mut line) = self.get_line()? {
            line.execute(self)?;
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

    pub fn get_builtin(&self, command: &str) -> Option<&Builtin> {
        self.builtins.get(command) 
    }

    pub fn clear_sources(&mut self) {
        self.sources.clear();
    }
}
