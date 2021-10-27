use anyhow::{anyhow, Context, Result};
use nix::sys::wait::wait;
use nix::unistd::{self, fork, ForkResult};
use std::ffi::CString;
use std::fmt;

use super::shell::Shell;
use super::sources::SourceKind;

mod word;
use word::Word;

// Represents a logical line given to the shell.
// Notably, a line can transcend physical lines by
// quoting, by backslash escaping a newline, and by
// terminating a line with a pipe operator.
#[derive(Clone)]
pub struct Line {
    rawline: String,
    line_num: usize,
    source: SourceKind,
    indentation: usize,
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

        Line {
            rawline,
            line_num,
            source,
            indentation: leading_spaces / 4,
        }
    }

    pub fn append(&mut self, text: String) {
        self.rawline.push_str(text.as_str());
    }

    pub fn source(&self) -> &SourceKind {
        &self.source
    }

    pub fn indentation(&self) -> usize {
        self.indentation
    }

    pub fn text(&self) -> String {
        self.rawline.clone()
    }

    // True if line is a complete logical line
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
        let substrings = break_line_into_words(&self.rawline)?;

        let mut words = Vec::<String>::new();

        for s in substrings {
            let mut word = Word::new(s)?;
            word.expand(smsh)?;
            word.separate()?;
            word.select()?;

            if !word.is_empty() {
                words.push(word.text().to_string());
            }
        }

        let strs = words.iter().map(|x| x.as_str()).collect();

        if words.is_empty() {
            return Ok(());
        }

        if smsh.push_user_function(&words) {
            Ok(())
        } else if let Some(f) = smsh.get_builtin(&words[0]) {
            f(smsh, strs)
        } else {
            match unsafe { fork()? } {
                ForkResult::Parent { child: _, .. } => {
                    wait()?;
                    Ok(())
                }
                ForkResult::Child => {
                    smsh.clear_sources();

                    let command = CString::new(strs[0])?;
                    let mut args = vec![];
                    for word in strs {
                        args.push(CString::new(word)?);
                    }
                    unistd::execvp(&command, &args).context(format!(
                        "Unable to execute external command `{}`",
                        &words[0]
                    ))?;
                    Ok(())
                }
            }
        }
    }
}

// Breaks line into words according to quoting rules.
// Quotes and braces are preserved, whitespace is removed
pub fn break_line_into_words(line: &str) -> Result<Vec<String>> {
    #[derive(PartialEq, Eq)]
    enum State {
        SingleQuoted,
        DoubleQuoted,
        Unquoted,
        Expansion(usize),
    }

    let mut words = Vec::<String>::new();
    let mut word = String::new();

    let mut state = State::Unquoted;

    for ch in line.chars() {
        match state {
            State::Unquoted => match ch {
                ' ' | '\n' | '\t' => {
                    if !word.is_empty() {
                        words.push(word);
                        word = String::new();
                    }
                }
                '\'' => {
                    word.push(ch);
                    state = State::SingleQuoted;
                }
                '\"' => {
                    word.push(ch);
                    state = State::DoubleQuoted;
                }
                '{' => {
                    word.push(ch);
                    state = State::Expansion(1);
                }
                _ => {
                    word.push(ch);
                }
            },
            State::SingleQuoted => {
                word.push(ch);
                if ch == '\'' {
                    words.push(word);
                    word = String::new();
                    state = State::Unquoted;
                } 
            }
            State::DoubleQuoted => {
                word.push(ch);
                if ch == '\"' {
                    word.push(ch);
                    words.push(word);
                    word = String::new();
                    state = State::Unquoted;
                } 
            }
            State::Expansion(n) => {
                if ch == '{' {
                    state = State::Expansion(n + 1);
                } else if ch == '}' {
                    state = State::Expansion(n - 1);
                }

                if state == State::Expansion(0) {
                    state = State::Unquoted
                }

                word.push(ch)
            }
        }
    }

    if !word.is_empty() {
        words.push(word);
    }

    match state {
        State::SingleQuoted => Err(anyhow!("Unmatched single quote.")),
        State::DoubleQuoted => Err(anyhow!("Unmatched double quote.")),
        State::Expansion(_) => Err(anyhow!("Unmatched brace.")),
        State::Unquoted => Ok(words),
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.source {
            SourceKind::Tty => {
                write!(f, "TTY line {}:\n{}", self.line_num, self.rawline)
            }
            SourceKind::Subshell => {
                write!(
                    f,
                    "Subshell Expansion line {}:\n{}",
                    self.line_num, self.rawline
                )
            }
            SourceKind::UserFunction(s) => {
                write!(
                    f,
                    "Function `{}` line number {}:\n{}",
                    s, self.line_num, self.rawline
                )
            }
            SourceKind::Script(s) => {
                write!(
                    f,
                    "Script `{}` line number {}:\n{}",
                    s, self.line_num, self.rawline
                )
            }
        }
    }
}
