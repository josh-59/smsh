use anyhow::{anyhow, Context, Result};
use nix::unistd::{self, fork, ForkResult};
use nix::sys::wait::wait;
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

        if words.len() == 0 {
            return Ok(());
        }

        if smsh.push_user_function(&words) {
            Ok(())
        } else if let Some(f) = smsh.get_builtin(&words[0]){
            f(smsh, strs)
        } else {
            match unsafe{fork()?} {
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
                    unistd::execvp(&command, &args)
                        .context(format!("Unable to execute external command `{}`", &words[0]))?;
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
    enum WordState {
        SingleQuoted,
        DoubleQuoted,
        Unquoted,
        Expansion(usize),
    }

    let mut words = Vec::<String>::new();
    let mut word = String::new();

    let mut state = WordState::Unquoted;

    for ch in line.chars() {
        match state {
            WordState::Unquoted => {
                match ch {
                    ' ' | '\n' | '\t' => {
                        if word.len() > 0 {
                            words.push(word);
                            word = String::new();
                        }
                    }
                    '\'' => {
                        word.push(ch);
                        state = WordState::SingleQuoted;
                    }
                    '\"' => {
                        word.push(ch);
                        state = WordState::DoubleQuoted;
                    }
                    '{' => {
                        word.push(ch);
                        state = WordState::Expansion(1);
                    }
                    _ => {
                        word.push(ch);
                    }
                }
            }
            WordState::SingleQuoted => {
                if ch == '\'' {
                    word.push(ch);
                    words.push(word);
                    word = String::new();
                    state = WordState::Unquoted;
                } else {
                    word.push(ch);
                }
            }
            WordState::DoubleQuoted => {
                if ch == '\"' {
                    word.push(ch);
                    words.push(word);
                    word = String::new();
                    state = WordState::Unquoted;
                } else {
                    word.push(ch);
                }
            }
            WordState::Expansion(n) => {
                if ch == '{' {
                    state = WordState::Expansion(n+1);
                } else if ch == '}' {
                    state = WordState::Expansion(n-1);
                }

                if state == WordState::Expansion(0) {
                    state = WordState::Unquoted
                }

                word.push(ch)
            }
        }
    }

    if word.len() > 0 {
        words.push(word);
    }

    match state {
        WordState::SingleQuoted => {
            Err(anyhow!("Unmatched single quote."))
        } 
        WordState::DoubleQuoted => {
            Err(anyhow!("Unmatched double quote."))
        } 
        WordState::Expansion(_) => {
            Err(anyhow!("Unmatched brace."))
        } 
        WordState::Unquoted => {
            Ok(words)
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
            SourceKind::Script(s) => {
                write!(f, "Script `{}` line number {}:\n{}", s, self.line_num, self.rawline)
            }
        }
    }
}

