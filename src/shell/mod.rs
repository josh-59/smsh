use crate::line::Line;
use crate::sources::{Sources, user_function::UserFunction, Source};
use anyhow::Result;
use nix::unistd;

use std::collections::HashMap;
use std::ffi::CString;
use std::process::exit;

mod state;
use state::State;
mod modules;
use modules::Builtin;
mod init;
use init::init;


pub struct Shell {
    state: State,
    sources: Sources,
    builtins: HashMap<&'static str, Builtin>,
    user_variables: HashMap<String, String>,
    user_functions: HashMap<String, UserFunction>,
}

impl Shell {
    pub fn new() -> Shell {
        init()
    }

    pub fn run(&mut self) -> Result<()> {
        while let Some(mut line) = self.get_line()? {
            line.execute(self)?;
        }

        Ok(())
    }

    fn get_line(&mut self) -> Result<Option<Line>> {
        self.sources.get_line()
    }

    pub fn get_block(&mut self) -> Result<Vec<Line>> {
        self.sources.get_block()
    }

    pub fn push_source(&mut self, source: Box<dyn Source>) {
        self.sources.push_source(source)
    }

    pub fn clear_sources(&mut self) {
        self.sources.clear_sources();
    }

    pub fn backtrace(&mut self) {
        self.sources.backtrace()
    }

    pub fn insert_user_variable(&mut self, key: String, val: String) {
        self.user_variables.insert(key, val);
    }

    pub fn get_user_variable(&mut self, key: &str) -> Option<String> {
        self.user_variables.get(key).cloned()
    }

    pub fn insert_user_function(&mut self, func: UserFunction) {
        self.user_functions.insert(func.name().to_string(), func);
    }

    pub fn get_builtin(&self, command: &str) -> Option<&Builtin> {
        self.builtins.get(command)
    }

    pub fn get_user_function(&self, name: &str) -> Option<UserFunction> {
        if let Some(func) = self.user_functions.get(name) {
            Some(func.clone())
        } else {
            None
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn execute_external_command(&mut self, args: Vec<String>) -> ! {
        if args.len() == 0 {
            exit(0);
        }

        let mut argv = vec![];
        for arg in args {
            let arg = match CString::new(arg) {
                Ok(x) => {
                    x
                }
                Err(e) => {
                    eprintln!("smsh (child): {}", e);
                    exit(1);
                }
            };

            argv.push(arg);
        }

        let _ = unistd::execvp(&argv[0], &argv);
        exit(1);

    }
}


