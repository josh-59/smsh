use anyhow::{anyhow, Result};
use nix::unistd::{close, dup, dup2, fork, ForkResult, pipe};
use nix::sys::wait::waitpid;

use std::os::unix::io::RawFd;

use super::Line;
use crate::Shell;
use crate::sources::user_function::UserFunction;
use crate::shell::modules::Builtin;

enum CommandKind {
    UserFunction(UserFunction),
    Builtin(Builtin),
    ExternalCommand(String)
}

pub struct Pipeline {
    elements: Vec<PipeElement>
}

impl Pipeline {
    pub fn new(line: &mut Line, smsh: &mut Shell) -> Result<Self> {

        let mut args = Vec::<String>::new();
        let mut elements = Vec::<PipeElement>::new();

        for word in line.words() {
            if word.is_pipe_operator() {
                let elem = PipeElement::new(args, smsh)?;
                args = Vec::<String>::new();
                elements.push(elem);
            } else {
                for arg in word.selected_text() {
                    if !arg.is_empty() {
                        args.push(arg.to_string());
                    }
                }
            }
        }

        if args.len() > 0 {
            let elem = PipeElement::new(args, smsh)?;
            elements.push(elem);
        }

        Ok( Pipeline {elements})
    }

    pub fn execute(&mut self, smsh: &mut Shell) -> Result<()> {
        if self.elements.len() == 0 {
            return Ok(());
        }

        let last = self.elements.len() - 1;

        let piped = if self.elements.len() > 1 {
            let stdin = dup(0)?;
            Some(stdin)
        } else {
            None
        };

        for elem in &mut self.elements[..last] {
            let (rd, wr) = pipe()?;

            match unsafe { fork()? } {
                ForkResult::Parent{child: _, ..} => {
                    close(wr)?;
                    close(0 as RawFd)?;
                    dup2(rd, 0)?;
                }

                ForkResult::Child => {
                    smsh.clear_sources();
                    close(rd)?;
                    close(1 as RawFd)?;
                    dup2(wr, 1)?;
                    close(wr)?;

                    elem.execute(smsh)?;
                }
            }
        }

        let mut last_elem = self.elements.pop().unwrap();

        if last_elem.is_external_command() {
            match unsafe { fork()? } {
                ForkResult::Parent{child: pid, ..} => {
                    waitpid(pid, None)?;
                }

                ForkResult::Child => {
                    smsh.clear_sources();
                    last_elem.execute(smsh)?;
                }
            }
        } else {
            last_elem.execute(smsh)?;
        }

        if let Some(stdin) = piped {
            close(0 as RawFd)?;
            dup2(stdin, 0)?;
        }

        Ok(())
    }
}

pub struct PipeElement {
    argv: Vec<String>,
    cmd_kind: CommandKind,
}

impl PipeElement {
    pub fn new(argv: Vec<String>, smsh: &mut Shell) -> Result<Self> {
        if argv.len() == 0 {
            return Err(anyhow!("Cannot create empty pipeline element"));
        }

        let cmd_kind = if let Some(f) = smsh.get_user_function(&argv[0]) {
            CommandKind::UserFunction(f)
        } else if let Some(f) = smsh.get_builtin(&argv[0]) {
            CommandKind::Builtin(*f)
        } else {
            CommandKind::ExternalCommand(argv[0].to_string())
        };

        Ok(PipeElement{argv, cmd_kind })
    }

    pub fn argv(&self) -> Vec<&str> {
        let mut strs = Vec::<&str>::new();

        for arg in &self.argv {
            if !arg.is_empty() {
                strs.push(arg.as_str())
            }
        }

        strs
    }

    pub fn is_external_command(&self) -> bool {
        match &self.cmd_kind {
            CommandKind::ExternalCommand(_) => {
                true
            }
            _ => {
                false
            }
        }

    }

    // Executes self in current shell context (that is, 'dumbly')
    pub fn execute(&mut self, smsh: &mut Shell) -> Result<()> {
        match &self.cmd_kind {
            CommandKind::UserFunction(f) => {
                smsh.push_source(f.clone().build_source());
                Ok(())
            }
            CommandKind::Builtin(b) => {
                b(smsh, self.argv())
            }
            CommandKind::ExternalCommand(cmd) => {
                let _ = smsh.execute_external_command(self.argv());
                Err(anyhow!("{}: Command not found", cmd))
            }
        }
    }
}
