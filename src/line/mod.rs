use anyhow::{anyhow, Result};
use nix::sys::wait::wait;
use nix::unistd::{fork, ForkResult};
use nix::sys::wait::WaitStatus;
use std::fmt;

use crate::shell::Shell;
use crate::sources::SourceKind;

mod word;
use word::Word;

#[derive(Clone, PartialEq, Eq, Debug)]
enum LineKind {
    Normal,
    If,
    Elif,
    Else,
    While,
    For,
}

// Represents a logical line given to the shell.
// A logical line can transcend physical lines by
// quoting, by backslash escaping a newline, and by
// terminating a line with a pipe operator.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Line {
    rawline: String,
    indentation: usize,
    source: SourceKind,
    line_num: usize,
    line_kind: LineKind,
}

impl Line {
    pub fn new(rawline: String, line_num: usize, source: SourceKind) -> Line {
        let (mut rawline, indentation) = get_indentation(rawline);

        while rawline.ends_with('\n') {
            rawline.pop();
        }

        let line_kind = get_line_kind(&rawline);

        Line {
            rawline,
            indentation,
            source,
            line_num,
            line_kind,
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

    pub fn is_empty(&self) -> bool {
        self.rawline.is_empty()
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

        let mut words = Vec::<String>::new();


        for s in self.get_words()? {
            let mut word = Word::new(s)?;
            word.expand(smsh)?;
            word.separate()?;
            word.select()?;

            for word in word.separated_text {
                if !word.is_empty() {
                    words.push(word);
                }
            }
        }

        let strs: Vec<&str> = words.iter().map(|x| x.as_str()).collect();

        if words.is_empty() {
            return Ok(());
        }

        if let Some(f) = smsh.get_user_function(strs[0]) {
            smsh.push_source(f.build_source());
            Ok(())
        } else if let Some(f) = smsh.get_builtin(&words[0]) {
            f(smsh, strs)
        } else {
            match unsafe { fork()? } {
                ForkResult::Parent { child: _, .. } => {
                    match wait()? {
                        WaitStatus::Exited(_pid, exit_status) => {
                            if exit_status > 0 {
                                Err(anyhow!("Unable to execute external command `{}`", &words[0]))
                            } else {
                                Ok(())
                            }
                        }
                        _ => {
                            Ok(())
                        }
                    }
                }
                ForkResult::Child => {
                    smsh.execute_external_command(words);
                }
            }
        }
    }

    // Breaks line into words according to quoting rules.
    // Quotes and braces are preserved, whitespace is removed
    fn get_words(&self) -> Result<Vec<String>> {
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

        for ch in self.rawline.chars() {
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
                        state = State::Unquoted;
                    } 
                }
                State::DoubleQuoted => {
                    word.push(ch);
                    if ch == '\"' {
                        state = State::Unquoted;
                    } 
                }
                State::Expansion(n) => {
                    word.push(ch);
                    if ch == '{' {
                        state = State::Expansion(n + 1);
                    } else if ch == '}' {
                        state = State::Expansion(n - 1);
                    }
    
                    if state == State::Expansion(0) {
                        state = State::Unquoted
                    }
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
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.source {
            SourceKind::Tty => {
                write!(f, "\tTTY line {}: {}", self.line_num, self.rawline)
            }
            SourceKind::Subshell => {
                write!(
                    f,
                    "\tSubshell Expansion line {}: {}",
                    self.line_num, self.rawline
                )
            }
            SourceKind::UserFunction(s) => {
                write!(
                    f,
                    "\tFunction `{}` line {}: {}",
                    s, self.line_num, self.rawline
                )
            }
            SourceKind::Script(s) => {
                write!(
                    f,
                    "\tScript `{}` line {}: {}",
                    s, self.line_num, self.rawline
                )
            }
        }
    }
}

fn get_line_kind(rawline: &str) -> LineKind {
    let mut first_word = String::new();

    for ch in rawline.chars() {
        if ch.is_whitespace() {
            break;
        }
        first_word.push(ch);
    }

    match first_word.as_str() {
        "if" => {
            LineKind::If
        }
        "elif" => {
            LineKind::Elif
        }
        "else" => {
            LineKind::Else
        }
        "while" => {
            LineKind::While
        }
        "for" => {
            LineKind::For
        } 
        _ => {
            LineKind::Normal
        }
    }
}

fn get_indentation(rawline: String) -> (String, usize) {
    let mut spaces: usize = 0;
    let mut leading_whitespace: usize = 0;
    let mut indentation: usize = 0;

    for ch in rawline.chars() {
        if ch == ' ' {
            leading_whitespace += 1;
            if spaces == 3 {
                indentation += 1;
                spaces = 0;
            } else {
                spaces += 1;
            }
        } else if ch == '\t' {
            leading_whitespace += 1;
            indentation += 1;
            spaces = 0;
        } else {
            break;
        }
    }
    
    (rawline[leading_whitespace..].to_string(), indentation)

}


#[cfg(test)]
mod test {
    use super::*;
    use crate::sources::SourceKind;

    #[test]
    fn new_line_test_1 () {
        let line = Line {
            rawline: "cmd".to_string(),
            indentation: 0,
            source: SourceKind::Subshell,
            line_num: 0,
        };

        assert_eq!(line, Line::new("cmd".to_string(), 0, SourceKind::Subshell));
    }

    #[test]
    fn new_line_test_2() {
        let line = Line {
            rawline: "cmd".to_string(),
            indentation: 1,
            source: SourceKind::Subshell,
            line_num: 0,
        };

        assert_eq!(line, Line::new("    cmd".to_string(), 0, SourceKind::Subshell));
    }

    #[test]
    fn new_line_test_3() {
        let line = Line {
            rawline: "cmd".to_string(),
            indentation: 0,
            source: SourceKind::Subshell,
            line_num: 0,
        };

        assert_eq!(line, Line::new("  cmd".to_string(), 0, SourceKind::Subshell));
    }
}
