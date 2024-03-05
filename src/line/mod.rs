use std::fmt;

use anyhow::{anyhow, Result};
use unicode_segmentation::UnicodeSegmentation;

use crate::constructs::{r#fn, r#for, r#if, r#let, r#while};
use crate::shell::Shell;
use crate::sources::SourceKind;

mod token;
use token::{get_tokens, Token};
mod pipeline;
use pipeline::Pipeline;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Construct {
    If,
    Elif,
    Else,
    FunctionDefinition,
    For,
    While,
    Let,
}

// Reflects the kind of command held by Line
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LineType {
    Normal,
    Empty,
    ShellConstruct(Construct),
}

// This struct identifies the source from which
// the line came, and the line number from within that
// source.  Used in error-reporting; should never be
// modified after creation.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LineID {
    pub source_kind: SourceKind,
    pub line_num: usize,
}

// A Line represents a logical line given to the shell.
// A logical line can transcend physical lines by
// quoting, by backslash escaping a newline, and by
// terminating a line with a pipe operator.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Line {
    line_id: LineID,
    raw_text: String, // Is a logical line
    tokens: Vec<Token>,
    line_type: LineType,
    indentation: usize,
}

impl Line {
    pub fn new(raw_text: String, line_num: usize, source_kind: SourceKind) -> Result<Line> {
        let line_id = LineID {
            source_kind,
            line_num,
        };

        // Break logical line into parts according to quoting rules
        let tokens = get_tokens(raw_text.as_str())?;

        let line_type = if tokens.len() > 0 {
            determine_line_type(tokens[0].text())
        } else {
            LineType::Empty
        };

        let indentation = determine_indentation(&raw_text);

        Ok(Line {
            line_id,
            raw_text,
            tokens,
            line_type,
            indentation,
        })
    }

    // Shell constructs use this
    pub fn argv(&self) -> Vec<&str> {
        let mut strs = Vec::<&str>::new();

        for token in &self.tokens {
            for s in token.selected_text() {
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
                pipeline.execute(smsh)
            }
            LineType::Empty => Ok(()),
            LineType::ShellConstruct(c) => match c {
                Construct::If | Construct::Elif | Construct::Else => r#if(smsh, self),
                Construct::FunctionDefinition => r#fn(smsh, self),
                Construct::For => r#for(smsh, self),
                Construct::Let => r#let(smsh, self),
                Construct::While => r#while(smsh, self),
            },
        }
    }

    pub fn expand(&mut self, smsh: &mut Shell) -> Result<()> {
        for token in &mut self.tokens {
            token.expand(smsh)?;
        }

        Ok(())
    }

    pub fn get_conditional(&self) -> Result<String> {
        let mut conditional = String::new();

        if self.tokens.len() < 2 {
            return Err(anyhow!("No conditional present"));
        }

        for s in &self.tokens[1..] {
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
        self.line_type == LineType::ShellConstruct(Construct::Elif)
    }

    pub fn is_else(&self) -> bool {
        self.line_type == LineType::ShellConstruct(Construct::Else)
    }

    pub fn raw_text(&self) -> &str {
        &self.raw_text
    }

    pub fn select(&mut self) -> Result<()> {
        for token in &mut self.tokens {
            token.select()?;
        }

        Ok(())
    }

    pub fn source(&self) -> &SourceKind {
        &self.line_id.source_kind
    }

    pub fn tokens(&self) -> &Vec<Token> {
        &self.tokens
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

// TODO: Remove! This functionality is not even close to being
// complete; should probably be elsewhere...
fn determine_line_type(first_token: &str) -> LineType {
    match first_token {
        "if" => LineType::ShellConstruct(Construct::If),
        "elif" => LineType::ShellConstruct(Construct::Elif),
        "else" => LineType::ShellConstruct(Construct::Else),
        "fn" => LineType::ShellConstruct(Construct::FunctionDefinition),
        "for" => LineType::ShellConstruct(Construct::For),
        "while" => LineType::ShellConstruct(Construct::While),
        "let" => LineType::ShellConstruct(Construct::Let),
        _ => LineType::Normal,
    }
}

fn determine_indentation(line: &str) -> usize {
    let mut spaces: usize = 0;
    let mut indentation = 0;

    for grapheme in line.graphemes(true) {
        match grapheme {
            " " => {
                if spaces == 3 {
                    indentation += 1;
                    spaces = 0;
                } else {
                    spaces += 1;
                }
            }
            "\t" => {
                indentation += 1;
                spaces = 0;
            }
            _ => {
                break;
            }
        }
    }

    indentation
}

/*
// Breaks `rawline` into parts according to quoting rules, yielding parts.
// Quotes and escapes are preserved; unquoted whitespace is removed
// Selection remains appended to part.
pub fn get_parts(rawline: &str) -> Result<Vec<&str>> {
    #[derive(PartialEq, Eq, Clone, Copy)]
    enum State {
        SingleQuoted,
        DoubleQuoted,
        Escaped,
        Unquoted,
        Expansion,
    }

    let mut part = String::new();
    let mut tokens = Vec::<Token>::new();

    let mut state = State::Unquoted;
    let mut escaped_state: Option<State> = None;

    for grapheme in rawline.graphemes(true) {
        match state {
            State::Unquoted => match grapheme {
                " " | "\t" => {
                    if !part.is_empty() {
                        tokens.push(Token::new(part)?);
                        part = String::new();
                    }
                }
                "\'" => {
                    part.push_str(grapheme);
                    state = State::SingleQuoted;
                }
                "\"" => {
                    part.push_str(grapheme);
                    state = State::DoubleQuoted;
                }
                "\\" => {
                    part.push_str(grapheme);
                    escaped_state = Some(State::Unquoted);
                    state = State::Escaped;
                }
                "{" => {
                    part.push_str(grapheme);
                    state = State::Expansion;
                }
                _ => {
                    part.push_str(grapheme);
                }
            },
            State::SingleQuoted => {
                part.push_str(grapheme);
                if grapheme == "\'" {
                    state = State::Unquoted;
                }
            }
            State::DoubleQuoted => {
                part.push_str(grapheme);
                if grapheme == "\"" {
                    state = State::Unquoted;
                } else if grapheme == "\\" {
                    escaped_state = Some(State::DoubleQuoted);
                    state = State::Escaped;
                }
            }
            State::Escaped => {
                part.push_str(grapheme);
                state = escaped_state.unwrap();
            }
            State::Expansion => {
                part.push_str(grapheme);
                if grapheme == "\\" {
                    escaped_state = Some(State::Expansion);
                    state = State::Escaped;
                } else if grapheme == "}" {
                    state = State::Unquoted;
                }
            }
        }
    }
    if !part.is_empty() {
        tokens.push(Token::new(part)?);
    }

    Ok(tokens)
}
*/
