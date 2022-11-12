use crate::line::Line;
use crate::sources::{Sources, SourceKind, user_function::UserFunction, Source, tty::Tty};
use anyhow::{anyhow, Result};
use nix::unistd::{self, fork, ForkResult};
use nix::sys::wait::{wait, WaitStatus};

use std::collections::HashMap;
use std::ffi::CString;
use std::process::exit;

mod state;
use state::State;
pub mod modules;
use modules::{Module, Builtin, load_module};
mod init;
use init::push_interactive_init_script;

pub struct Shell {
    state: State,
    sources: Sources,
    builtins: HashMap<&'static str, Builtin>,
    user_variables: HashMap<String, String>,
    user_functions: HashMap<String, UserFunction>,
}

impl Shell {

    // This function should never fail, so that
    // a user of smsh always gets into its main loop.
    pub fn new() -> Shell {
        let state = State::new();
        let sources = Sources::new();
        let builtins = HashMap::<&'static str, Builtin>::new();
        let user_variables = HashMap::<String, String>::new();
        let user_functions = HashMap::<String, UserFunction>::new();
    
        let mut smsh = Shell {
            state,
            sources,
            builtins,
            user_variables,
            user_functions,
        };

        load_module(&mut smsh, Module::Core);

        if smsh.state.is_interactive() {
            smsh.push_source(Tty::build_source());
            push_interactive_init_script(&mut smsh);
        }

        smsh
    }

    pub fn run(&mut self) -> Result<()> {
        while let Some(mut line) = self.get_line()? {
            line.expand(self)?;
            line.separate()?;
            line.select()?;
            line.execute(self)?;
        }

        Ok(())
    }

    // TODO: Overhaul source constructs s.t. this fn deals with sources
    pub fn get_line(&mut self) -> Result<Option<Line>> {
        if self.sources.len() == 0 {
            Ok(None)
        } else {
            self.sources.get_line()
        }
    }

    pub fn get_block(&mut self, source_kind: &SourceKind, indent: usize) -> Result<Vec<Line>> {
        self.sources.get_block(source_kind, indent)
    }

    pub fn push_source(&mut self, source: Box<dyn Source>) {
        self.sources.push_source(source)
    }

    pub fn push_front(&mut self, line: Line) {
        self.sources.push_front(line);
    }

    pub fn push_back(&mut self, line:Line) {
        self.sources.push_back(line);
    }

    pub fn push_block(&mut self, lines: Vec<Line>) {
        self.sources.push_block(lines);
    }

    pub fn clear_sources(&mut self) {
        self.sources.clear();
    }

    fn is_interactive(&mut self) -> bool {
        self.state.is_interactive()
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

    pub fn set_rv(&mut self, rv: i32) {
        self.state.rv = rv;
    }

    pub fn rv(&self) -> i32 {
        self.state.rv
    }

    // Executes `line` in a subshell environment, waits for it
    // and collects its return value.
    pub fn execute_subshell(&mut self, line: &str) -> Result<bool> {
        match unsafe { fork()? } {
            ForkResult::Parent { child: _, .. } => {
                match wait()? {
                    WaitStatus::Exited(_pid, exit_status) => {
                        if exit_status == 0 {
                            Ok(true)
                        } else {
                            Ok(false)
                        }
                    }
                    _ => {
                        Err(anyhow!("wait: Failed to wait on subshell with line {}", line))
                    }
                }
            }

            // TODO: This error should propogate backwards through the call stack,
            // as with any other error.
            ForkResult::Child => {
                self.clear_sources();
                let line = Line::new(line.to_string(), 0, SourceKind::Subshell)?;
                self.push_front(line);

                while let Err(e) = self.run() {
                    eprintln!("smsh (subshell): {}", e);
                }

                std::process::exit(self.state.rv);
            }
        }
    }

    pub fn execute_external_command(&mut self, args: Vec<&str>) -> Result<()> {
        let mut argv = Vec::<CString>::new();

        for arg in args {
            let c_arg = match CString::new(arg) {
                Ok(x) => {
                     x
                }
                Err(e) => {
                    eprintln!("smsh (child): {}", e);
                    exit(1);
                }
            };

            argv.push(c_arg);
        }

        let _ = unistd::execvp(&argv[0], &argv);
        Ok(())
    }
}



