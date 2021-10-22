use anyhow::Result;
use nix::unistd::{self, fork, ForkResult};
use nix::sys::wait::wait;
use std::ffi::CString;
use std::fmt;

use super::shell::Shell;

pub struct Line {
    pub rawline: String,
}

impl Line {
    pub fn new(s: String) -> Result<Line> {
        let rawline = s.trim().to_string();

        Ok( Line { rawline } )
    }

    pub fn execute(&mut self, smsh: &mut Shell) -> Result<()> {
        let words: Vec::<String> = self.rawline
                    .split_whitespace()
                    .map(|x| x.to_string())
                    .collect();

        if words.len() == 0 {
            return Ok(());
        }

        if let Some(f) = smsh.get_builtin(&words[0]) {
            f(smsh, words)
        } else {
            match unsafe{fork()?} {
                ForkResult::Parent { child: _, .. } => {
                    wait()?;
                    Ok(())
                }
                ForkResult::Child => {
                    let command = CString::new(words[0].clone())?;
                    let mut args = vec![];
                    for word in words {
                        args.push(CString::new(word)?);
                    }
                    unistd::execvp(&command, &args)?;
                    Ok(())
                }
            }
        }
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rawline)
    }
}
