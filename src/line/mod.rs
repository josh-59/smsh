use std::fmt;

use anyhow::{anyhow, Result};
use unicode_segmentation::UnicodeSegmentation;

use crate::shell::Shell;
use crate::sources::SourceKind;
use crate::constructs::{r#if, r#fn, r#for, r#while, r#let};

mod word;
use word::Word;
mod pipeline;
use pipeline::Pipeline;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LineKind {
    Normal,
    If,
    Elif,
    Else,
    FunctionDefinition,
    For,
    While,
    Let,
    Empty,
}

// Should only ever be read-only
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LineID {
    pub source_kind: SourceKind,
    pub line_num: usize,
}

// Represents a logical line given to the shell.
// A logical line can transcend physical lines by
// quoting, by backslash escaping a newline, and by
// terminating a line with a pipe operator.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Line {
    rawline: String,    // Original string passed to Line. 
    line_kind: LineKind,
    line_id: LineID,
    words: Vec<Word>,
    indentation: usize,
}

impl Line {
    pub fn new(rawline: String, line_num: usize, source_kind: SourceKind) -> Result<Line> {

        let line_id = LineID {
            source_kind,
            line_num, 
        };

        let indentation = get_indentation(&rawline);

        // Get words
        let mut words = Vec::<Word>::new();
        for word in get_words(&rawline)? {
            words.push(Word::new(word)?);
        }

        let line_kind = if words.len() > 0 {
            get_line_kind(&words[0])
        } else {
            LineKind::Empty
        };

        Ok( Line {
            rawline,
            line_kind,
            line_id,
            words,
            indentation,
        } )
    }

    // Shell constructs use this
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
                let mut pipeline = Pipeline::new(self, smsh)?;

                if smsh.state().no_exec() {
                    Ok(())
                } else {
                    pipeline.execute(smsh)
                }
            }
            LineKind::If | LineKind::Elif | LineKind::Else => {
                r#if(smsh, self)
            }
            LineKind::FunctionDefinition => {
                r#fn(smsh, self)
            }
            LineKind::For => {
                r#for(smsh, self)
            }
            LineKind::While => {
                r#while(smsh, self)
            }
            LineKind::Let => {
                r#let(smsh, self)
            }
            LineKind::Empty => {
                Ok(())
            }
        }
    }

    pub fn expand(&mut self, smsh: &mut Shell) -> Result<()> {
        for word in &mut self.words {
            word.expand(smsh)?;
        }

        Ok(())
    }

    pub fn get_conditional(&self) -> Result<String> {
        let mut conditional = String::new();

        if self.words.len() < 2 {
            return Err(anyhow!("No conditional present"));
        }

        for s in &self.words[1..] {
            conditional.push_str(s.text());
            conditional.push(' ');
        }

        conditional.pop();  // Remove trailing whitespace

        Ok(conditional)
    }

    pub fn identifier(&self) -> &LineID {
        &self.line_id
    }

    pub fn indentation(&self) -> usize {
        self.indentation
    }

    pub fn is_elif(&self) -> bool {
        self.line_kind == LineKind::Elif
    }

    pub fn is_else(&self) -> bool {
        self.line_kind == LineKind::Else
    }

    pub fn rawline(&self) -> &str {
        &self.rawline
    }
    
    pub fn source(&self) -> &SourceKind {
        &self.line_id.source_kind
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

    pub fn words(&self) -> &Vec<Word> {
        &self.words
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.line_id.source_kind {
            SourceKind::Tty => {
                write!(f, "\tTTY line {}: {}", self.line_id.line_num, self.rawline)
            }
            SourceKind::Subshell => {
                write!(
                    f,
                    "\tSubshell Expansion line {}: {}",
                    self.line_id.line_num, self.rawline
                )
            }
            SourceKind::UserFunction(s) => {
                write!(
                    f,
                    "\tFunction `{}` line {}: {}",
                    s, self.line_id.line_num, self.rawline
                )
            }
            SourceKind::Script(s) => {
                write!(
                    f,
                    "\tScript `{}` line {}: {}",
                    s, self.line_id.line_num, self.rawline
                )
            }
        }
    }
}

fn get_line_kind(word: &Word) -> LineKind {
    match word.text() {
        "if" => {
            LineKind::If
        }
        "elif" => {
            LineKind::Elif
        }
        "else" => {
            LineKind::Else
        }
        "fn" => {
            LineKind::FunctionDefinition
        }
        "for" => {
            LineKind::For
        }
        "while" => {
            LineKind::While
        }
        "let" => {
            LineKind::Let
        }
        _ => {
            LineKind::Normal
        }
    }
}

// Breaks rawline into words according to quoting rules.
// Quotes and braces are preserved, whitespace is removed
// TODO Implement escape via backslash \
fn get_words(rawline: &str) -> Result<Vec<String>> {
    #[derive(PartialEq, Eq)]
    enum State {
        SingleQuoted,
        DoubleQuoted,
        Unquoted,
        Expansion(usize, usize), // Index, depth
    }

    let mut words = Vec::<String>::new();
    let mut word = String::new();

    let mut state = State::Unquoted;

    for (i, grapheme) in rawline.graphemes(false).enumerate() {
        match state {
            State::Unquoted => match grapheme {
                " " | "\t" => {
                    if !word.is_empty() {
                        words.push(word);
                        word = String::new();
                    }
                }
                "\'" => {
                    word.push_str(grapheme);
                    state = State::SingleQuoted;
                }
                "\"" => {
                    word.push_str(grapheme);
                    state = State::DoubleQuoted;
                }
                "{" => {
                    word.push_str(grapheme);
                    state = State::Expansion(i, 1);
                }
                _ => {
                    word.push_str(grapheme);
                }
            },
            State::SingleQuoted => {
                word.push_str(grapheme);
                if grapheme == "\'" {
                    state = State::Unquoted;
                } 
            },
            State::DoubleQuoted => {
                word.push_str(grapheme);
                if grapheme == "\"" {
                    state = State::Unquoted;
                } 
            },
            State::Expansion(_, n) => {
                word.push_str(grapheme);
                if grapheme == "{" {
                    state = State::Expansion(i, n + 1);
                } else if grapheme == "}" {
                    state = State::Expansion(i, n - 1);
                }

                if state == State::Expansion(i, 0) {
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
        State::Expansion(i, _) => Err(anyhow!(
            format!("{}\n{:>width$} Unmatched expansion brace", rawline, "^", width = "smsh: ".len() + i + 1))),
        State::Unquoted => Ok(words),
    }
}


fn get_indentation(rawline: &str) -> usize {
    let mut spaces: usize = 0;
    let mut indentation = 0;

    for ch in rawline.chars() {
        if ch == ' ' {
            if spaces == 3 {
                indentation += 1;
                spaces = 0;
            } else {
                spaces += 1;
            }
        } else if ch == '\t' {
            indentation += 1;
            spaces = 0;
        } else {
            break;
        }
    }

    indentation
}

