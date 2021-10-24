use anyhow::Result;
use nix::unistd::getuid;
use super::sources::{Source, 
    tty::TTY, 
    user_function::UserFunction,
    BufferSource};
use super::line::Line;

use std::collections::HashMap;

mod modules;
use modules::*;

pub struct Shell {
    sources: Vec<Box<dyn Source>>,
    builtins: HashMap<&'static str, Builtin>,
    user_variables: HashMap<String, String>,
    user_functions: HashMap<String, UserFunction>,
}

impl Shell {
    pub fn new() -> Result<Shell> {
        let sources = vec![TTY::new()?];
        let builtins = HashMap::<&'static str, Builtin>::new();
        let user_variables = HashMap::<String, String>::new();
        let user_functions = HashMap::<String, UserFunction>::new();

        let mut smsh = Shell { 
            sources, 
            builtins, 
            user_variables,
            user_functions,
        };

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
            let prompt = if getuid().is_root() {
                "# "
            } else {
                "$ "
            };

            if let Some(line) = source.get_line(&prompt)? {
                self.sources.push(source);
                Ok(Some(line))
            } else {
                self.get_line()
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_block(&mut self) -> Result<Vec<Line>> {
        let mut lines = Vec::<Line>::new();

        let first_line = self.get_line()?.unwrap();
        let source = first_line.source().clone();
        let indent = first_line.indentation();
        lines.push(first_line);

        while let Some(line) = self.get_line()? {
            if *line.source() == source && line.indentation() == indent {
                lines.push(line);
            } else {
                self.push_source(BufferSource::new(vec![line]));
                break;
            }
        }

        Ok(lines)
    }

    pub fn get_builtin(&self, command: &str) -> Option<&Builtin> {
        self.builtins.get(command) 
    }

    pub fn push_source(&mut self, source: Box<dyn Source>) {
        self.sources.push(source)
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

    pub fn insert_user_function(&mut self, func: UserFunction) {
        self.user_functions.insert(func.name().to_string(), func);
    }

    pub fn push_user_function(&mut self, args: Vec<String>) -> bool {
        if args.is_empty() {
            false
        } else if let Some(func) = self.user_functions.get(&args[0]) {
            self.sources.push(func.build_source());
            true
        } else {
            false
        }
    }
}
