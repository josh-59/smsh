use crate::line::Line;
use crate::sources::{user_function::UserFunction, InputSource};
use anyhow::Result;
use nix::unistd;

use std::collections::HashMap;
use std::ffi::CString;
use std::process::exit;

mod state;
use state::State;
mod modules;
use modules::*;
mod init;
use init::init;


pub struct Shell {
    state: State,
    sources: Vec<Box<dyn InputSource>>,
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
        if let Some(mut source) = self.sources.pop() {
            if let Some(line) = source.get_line(None)? {
                self.sources.push(source);
                Ok(Some(line))
            } else {
                self.get_line()
            }
        } else {
            Ok(None)
        }
    }

    // TODO: Resolve commented line below!
    pub fn get_block(&mut self) -> Result<Vec<Line>> {
        let mut lines = Vec::<Line>::new();

        if let Some(first_line) = self.get_line()? {
            let source = first_line.source().clone();
            let indent = first_line.indentation();
            lines.push(first_line);

            while let Some(line) = self.get_line()? {
                if *line.source() == source && line.indentation() == indent {
                    lines.push(line);
                } else if line.is_empty() {
                    continue;
                } else {
                    // self.push_source(BufferSource::build_source(vec![line]));
                    break;
                }
            }
        }

        Ok(lines)
    }

    pub fn push_source(&mut self, source: Box<dyn InputSource>) {
        self.sources.push(source)
    }

    pub fn clear_sources(&mut self) {
        self.sources.clear();
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

    pub fn backtrace(&mut self) {
        while let Some(mut source) = self.sources.pop() {
            if !source.is_faux_source() && !source.is_tty() {
                source.print_error();
            }

            if source.is_tty() {
                self.sources.push(source);
                break;
            }
        }
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


