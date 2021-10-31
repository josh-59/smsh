use anyhow::{anyhow, Result};
use std::fmt;

use crate::shell::Shell;
use crate::sources::SourceKind;
use crate::constructs::r#if;

mod word;
use word::Word;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LineKind {
    Normal,
    If,
    Elif,
    Else,
}

// Represents a logical line given to the shell.
// A logical line can transcend physical lines by
// quoting, by backslash escaping a newline, and by
// terminating a line with a pipe operator.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Line {
    rawline: String,
    line_kind: LineKind,
    indentation: usize,
    source: SourceKind,
    line_num: usize,
    words: Vec<Word>,
}

impl Line {
    pub fn new(rawline: String, line_num: usize, source: SourceKind) -> Result<Line> {
        let (mut rawline, indentation) = get_indentation(rawline);

        while rawline.ends_with('\n') {
            rawline.pop();
        }

        let mut words = Vec::<Word>::new();

        for word in get_words(rawline.as_str())? {
            if !word.is_empty() {
                words.push(Word::new(word)?);
            }
        }

        let line_kind = get_line_kind(&words);

        Ok( Line {
            rawline,
            line_kind,
            indentation,
            source,
            line_num,
            words,
        } )
    }

    pub fn indentation(&self) -> usize {
        self.indentation
    }

    pub fn is_if(&self) -> bool {
        self.line_kind == LineKind::If
    }

    pub fn is_elif(&self) -> bool {
        self.line_kind == LineKind::Elif
    }

    pub fn is_else(&self) -> bool {
        self.line_kind == LineKind::Else
    }

    pub fn source(&self) -> &SourceKind {
        &self.source
    }

    pub fn text(&self) -> String {
        self.rawline.clone()
    }

    pub fn expand(&mut self, smsh: &mut Shell) -> Result<()> {
        for word in &mut self.words {
            word.expand(smsh)?;
        }

        Ok(())
    }

    pub fn separate(&mut self) -> Result<()> {
        for word in &mut self.words {
            word.separate()?;
        }

        Ok(())
    }

    pub fn select(&mut self) -> Result<()> {
        for word in &mut self.words {
            word.select()?;
        }

        Ok(())
    }

    // TODO: Move this to PipeElem struct
    pub fn argv(&self) -> Vec<&str> {
        let mut strs = Vec::<&str>::new();

        for word in &self.words {
            for s in word.selected_text() {
                if !s.is_empty() {
                    strs.push(s);
                }
            }
        }

        strs
    }

    pub fn execute(&mut self, smsh: &mut Shell) -> Result<()> {
        match &self.line_kind {
            LineKind::Normal => {
                let strs = self.argv();
        
                if strs.is_empty() {
                    return Ok(());
                }
    
                if let Some(f) = smsh.get_user_function(strs[0]) {
                    smsh.push_source(f.build_source());
                    Ok(())
                } else if let Some(f) = smsh.get_builtin(strs[0]) {
                    f(smsh, self)?;
                    Ok(())
                } else {
                    smsh.execute_external_command(strs)?;
                    Ok(())
                }
            }
            LineKind::If => {
                r#if(smsh, self)?;
                Ok(())
            }
            _ => {
                Ok(())
            }
        }
    }

    pub fn get_conditional(&self) -> Result<String> {
        let mut conditional = String::new();

        for s in &self.words[1..] {
            conditional.push_str(s.text());
            conditional.push(' ');
        }

        conditional.pop();  // Remove trailing whitespace

        if conditional.ends_with(':') {
            conditional.pop();  // Remove trailing semicolon
            Ok(conditional)
        } else {
            Err(anyhow!("Improperly formed conditional: No trailing semicolon present"))
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

fn get_line_kind(words: &Vec<Word>) -> LineKind {
    if words.len() == 0 {
        LineKind::Normal
    } else if words[0].text() == "if" {
        LineKind::If
    } else if words[0].text() == "elif" {
        LineKind::Elif
    } else if words[0].text() == "else:" {
        LineKind::Else
    } else {
        LineKind::Normal
    }
}

// Breaks rawline into words according to quoting rules.
// Quotes and braces are preserved, whitespace is removed
fn get_words(rawline: &str) -> Result<Vec<String>> {
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

    for ch in rawline.chars() {
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


