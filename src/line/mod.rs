use std::fmt;

use anyhow::{anyhow, Result};
use unicode_segmentation::UnicodeSegmentation;

use crate::constructs::{r#fn, r#for, r#if, r#let, r#while};
use crate::shell::Shell;
use crate::sources::SourceKind;

mod word;
use word::Word;
mod pipeline;
use pipeline::Pipeline;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LineType {
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
    raw_text: String, // Original string passed to Line.
    line_type: LineType,
    line_id: LineID,
    words: Vec<Word>,
    indentation: usize,
}

impl Line {
    pub fn new(raw_text: String, line_num: usize, source_kind: SourceKind) -> Result<Line> {
        let line_id = LineID {
            source_kind,
            line_num,
        };

        let indentation = get_indentation(&raw_text);

        // Get parts of line
        let mut words = Vec::<Word>::new();
        for word in get_parts(&raw_text)? {
            words.push(Word::new(word)?);
        }

        let line_type = if words.len() > 0 {
            get_line_type(&words[0])
        } else {
            LineType::Empty
        };

        Ok(Line {
            raw_text,
            line_type,
            line_id,
            words,
            indentation,
        })
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
        match &self.line_type {
            LineType::Normal => {
                let mut pipeline = Pipeline::new(self, smsh)?;

                if smsh.state().no_exec() {
                    Ok(())
                } else {
                    pipeline.execute(smsh)
                }
            }
            LineType::If | LineType::Elif | LineType::Else => r#if(smsh, self),
            LineType::FunctionDefinition => r#fn(smsh, self),
            LineType::For => r#for(smsh, self),
            LineType::While => r#while(smsh, self),
            LineType::Let => r#let(smsh, self),
            LineType::Empty => Ok(()),
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

        conditional.pop(); // Remove trailing whitespace

        Ok(conditional)
    }

    pub fn identifier(&self) -> &LineID {
        &self.line_id
    }

    pub fn indentation(&self) -> usize {
        self.indentation
    }

    pub fn is_elif(&self) -> bool {
        self.line_type == LineType::Elif
    }

    pub fn is_else(&self) -> bool {
        self.line_type == LineType::Else
    }

    pub fn raw_text(&self) -> &str {
        &self.raw_text
    }
    pub fn source(&self) -> &SourceKind {
        &self.line_id.source_kind
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
                write!(f, "\tTTY line {}: {}", self.line_id.line_num, self.raw_text)
            }
            SourceKind::Subshell => {
                write!(
                    f,
                    "\tSubshell Expansion line {}: {}",
                    self.line_id.line_num, self.raw_text
                )
            }
            SourceKind::UserFunction(s) => {
                write!(
                    f,
                    "\tFunction `{}` line {}: {}",
                    s, self.line_id.line_num, self.raw_text
                )
            }
            SourceKind::Script(s) => {
                write!(
                    f,
                    "\tScript `{}` line {}: {}",
                    s, self.line_id.line_num, self.raw_text
                )
            }
        }
    }
}

// TODO: This function should be changed:  Beginning with an "if" statement does not
// imply that the line is correctly-formatted
fn get_line_type(word: &Word) -> LineType {
    match word.text() {
        "if" => LineType::If,
        "elif" => LineType::Elif,
        "else" => LineType::Else,
        "fn" => LineType::FunctionDefinition,
        "for" => LineType::For,
        "while" => LineType::While,
        "let" => LineType::Let,
        _ => LineType::Normal,
    }
}

// Breaks rawline into parts according to quoting rules.
// Quotes and braces are preserved, (unquoted) whitespace is removed
// TODO Implement escape via backslash \
fn get_parts(rawline: &str) -> Result<Vec<String>> {
    #[derive(PartialEq, Eq)]
    enum State {
        SingleQuoted(usize), // usize reflects location of found quote
        DoubleQuoted(usize),
        Unquoted,
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
                    state = State::SingleQuoted(i);
                }
                "\"" => {
                    word.push_str(grapheme);
                    state = State::DoubleQuoted(i);
                }

                _ => {
                    word.push_str(grapheme);
                }
            },
            State::SingleQuoted(j) => {
                word.push_str(grapheme);
                if grapheme == "\'" {
                    state = State::Unquoted;
                }
            }
            State::DoubleQuoted(j) => {
                word.push_str(grapheme);
                if grapheme == "\"" {
                    state = State::Unquoted;
                }
            }
        }
    }
    if !word.is_empty() {
        words.push(word);
    }

    match state {
        State::SingleQuoted(i) => Err(anyhow!("Unmatched single quote.")),
        State::DoubleQuoted(i) => Err(anyhow!("Unmatched double quote.")),
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
