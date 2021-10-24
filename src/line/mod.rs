use anyhow::Result;
use nix::unistd::{self, fork, ForkResult};
use nix::sys::wait::wait;
use std::ffi::CString;
use std::fmt;

use super::shell::Shell;
use super::sources::SourceKind;

mod word;
use word::get_words_from_str;

// Represents a logical line given to the shell.
// Notably, a line can transcend physical lines by
// quoting, by backslash escaping a newline, and by
// terminating a line with a pipe operator.
#[derive(Clone)]
pub struct Line {
    rawline: String,
    line_num: usize,
    source: SourceKind,
    indentation: usize
}

impl Line {
    pub fn new(rawline: String, line_num: usize, source: SourceKind) -> Line {
        let mut leading_spaces: usize = 0;
        for ch in rawline.chars() {
            if ch == ' ' {
                leading_spaces += 1;
            } else {
                break;
            }
        }

        let rawline = rawline[leading_spaces..].to_string();

        Line { rawline, line_num, source, indentation: leading_spaces / 4 } 
    }

    pub fn append(&mut self, text: String) {
        self.rawline.push_str(text.as_str());
    }

    pub fn source(&self) -> &SourceKind {
        &self.source
    }

    pub fn line_num(&self) -> usize {
        self.line_num
    }

    pub fn indentation(&self) -> usize {
        self.indentation
    }

    pub fn text(&self) -> String {
        self.rawline.clone()
    }

    // True if line is a complete logical line
    // Assumes trailing newline has been removed.
    pub fn is_complete(&self) -> bool {
        let mut single_quoted = false;
        let mut double_quoted = false;
        let mut escaped = false;

        for ch in self.rawline.chars() {
            if escaped {
                escaped = false;
            } else {
                match ch {
                    '\\' => {
                        escaped = true;
                    }
                    '\'' => {
                        single_quoted = !single_quoted;
                    }
                    '\"' => {
                        double_quoted = !double_quoted;
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }

        !(single_quoted || double_quoted || escaped)
    }

    pub fn execute(&mut self, smsh: &mut Shell) -> Result<()> {

        let mut words = get_words_from_str(&self.rawline)?;

        for word in &mut words {
            word.expand(smsh)?;
        }

        if words.len() == 0 {
            return Ok(());
        }

        //XXX
        let words_string = words.iter().map(|x| x.text().to_string()).collect();

        if smsh.push_user_function(words_string) {
            Ok(())
        } else if let Some(f) = smsh.get_builtin(words[0].text()) {
            let strings = words.iter().map(|x| x.text().to_string()).collect();
            f(smsh, strings)
        } else {
            match unsafe{fork()?} {
                ForkResult::Parent { child: _, .. } => {
                    wait()?;
                    Ok(())
                }
                ForkResult::Child => {
                    smsh.clear_sources();
                    let command = CString::new(words[0].text().clone())?;
                    let mut args = vec![];
                    for word in words {
                        args.push(CString::new(word.text())?);
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
        match &self.source {
            SourceKind::TTY => {
                write!(f, "TTY line {}:\n{}", self.line_num, self.rawline)
            }
            SourceKind::Subshell => {
                write!(f, "Subshell Expansion line {}:\n{}", self.line_num, self.rawline)
            }
            SourceKind::UserFunction(s) => {
                write!(f, "Function `{}` line number {}:\n{}", s, self.line_num, self.rawline)
            }
        }
    }
}

