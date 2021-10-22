use anyhow::Result;
use nix::unistd::{self, fork, ForkResult};
use nix::sys::wait::wait;
use std::ffi::CString;
use std::fmt;

use super::shell::Shell;
use super::source::SourceKind;

pub struct Line {
    rawline: String,
    line_num: usize,
    source: SourceKind,
}

impl Line {
    pub fn new(rawline: String, line_num: usize, source: SourceKind) -> Result<Line> {
        let rawline = rawline.trim().to_string();

        Ok( Line { rawline, line_num, source } )
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
        match self.source {
            SourceKind::TTY => {
                write!(f, "TTY line {}:\n{}", self.line_num, self.rawline)
            }
        }
    }
}
