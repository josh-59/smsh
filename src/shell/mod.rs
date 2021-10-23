use anyhow::Result;
use super::sources::{Source, tty::TTY};
use super::line::Line;

use std::collections::HashMap;

mod modules;
use modules::*;

pub struct Shell {
    sources: Vec<Box<dyn Source>>,
    builtins: HashMap<&'static str, Builtin>,
    user_variables: HashMap<String, String>,
}

impl Shell {
    pub fn new() -> Result<Shell> {
        let tty = TTY::new()?;
        let sources = vec![tty];

        let builtins = HashMap::<&'static str, Builtin>::new();
        let user_variables = HashMap::<String, String>::new();

        let mut smsh = Shell { sources, builtins, user_variables };

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

    pub fn insert_user_variable(&mut self, key: String, val: String) {
        self.user_variables.insert(key, val);
    }

    pub fn get_user_variable(&mut self, key: &str) -> Option<String> {
        if let Some(val) = self.user_variables.get(key) {
            Some(val.clone())
        } else {
            None
        }
    }
}
