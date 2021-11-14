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
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LineIdentifier {
    source: SourceKind,
    line_num: usize,
    indentation: usize,
}

impl LineIdentifier {
    pub fn source(&self) -> SourceKind {
        self.source.clone()
    }

    pub fn line_num(&self) -> usize {
        self.line_num
    }

    pub fn indentation(&self) -> usize {
        self.indentation
    }
}

// Represents a logical line given to the shell.
// A logical line can transcend physical lines by
// quoting, by backslash escaping a newline, and by
// terminating a line with a pipe operator.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Line {
    rawline: String,    // Original string passed to Line. 
                        // Does not include trailing newlines 
    line_kind: LineKind,
    line_identifier: LineIdentifier,
    words: Vec<Word>,
}

impl Line {
    pub fn new(mut rawline: String, line_num: usize, source: SourceKind) -> Result<Line> {

        // Finalize rawline
        while rawline.ends_with('\n') {
            rawline.pop();
        }

        // Build LineIdentifier
        let (mut text, indentation) = get_indentation(rawline.clone());
        let line_identifier = LineIdentifier {
            source,
            line_num, 
            indentation,
        };

        // Assert line type
        let line_kind = get_line_kind(text.as_str())?;
        if line_kind != LineKind::Normal && line_kind != LineKind::Let {
            text.pop(); // Remove trailing colon
        }

        // Get words
        let mut words = Vec::<Word>::new();
        for word in get_words(text.as_str())? {
            if !word.is_empty() {
                words.push(Word::new(word)?);
            }
        }

        Ok( Line {
            rawline,
            line_kind,
            line_identifier,
            words,
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

                pipeline.execute(smsh)
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

    pub fn identifier(&self) -> LineIdentifier {
        self.line_identifier.clone()
    }

    pub fn indentation(&self) -> usize {
        self.line_identifier.indentation()
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

    pub fn rawline(&self) -> &str {
        &self.rawline
    }
    
    pub fn source(&self) -> &SourceKind {
        &self.line_identifier.source
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
        match &self.line_identifier.source {
            SourceKind::Tty => {
                write!(f, "\tTTY line {}: {}", self.line_identifier.line_num, self.rawline)
            }
            SourceKind::Subshell => {
                write!(
                    f,
                    "\tSubshell Expansion line {}: {}",
                    self.line_identifier.line_num, self.rawline
                )
            }
            SourceKind::UserFunction(s) => {
                write!(
                    f,
                    "\tFunction `{}` line {}: {}",
                    s, self.line_identifier.line_num, self.rawline
                )
            }
            SourceKind::Script(s) => {
                write!(
                    f,
                    "\tScript `{}` line {}: {}",
                    s, self.line_identifier.line_num, self.rawline
                )
            }
        }
    }
}

// TODO: Maybe hash these somehow?
fn get_line_kind(text: &str) -> Result<LineKind> {
    if text.starts_with("if") {
        if text.ends_with(':') {
            Ok(LineKind::If)
        } else {
            Err(anyhow!("Improperly formed if"))
        }
    } else if text.starts_with("elif") {
        if text.ends_with(':') {
            Ok(LineKind::Elif)
        } else {
            Err(anyhow!("Improperly formed elif"))
        }
    } else if text == "else:" {
        Ok(LineKind::Else)
    } else if text.starts_with("fn") {
        if text.ends_with(":") {
            Ok(LineKind::FunctionDefinition)
        } else {
            Err(anyhow!("Improperly formed function definition"))
        }
    } else if text.starts_with("for") {
        if text.ends_with(":") {
            Ok(LineKind::For)
        } else {
            Err(anyhow!("Improperly formed `for` construct\n{}", text))
        }
    } else if text.starts_with("while") {
        if text.ends_with(":") {
            Ok(LineKind::While)
        } else {
            Err(anyhow!("Improperly formed `while` construct\n{}", text))
        }
    } else if text.starts_with("let") {
        Ok(LineKind::Let)
    } else {
        Ok(LineKind::Normal)
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
        Expansion(usize, usize), // Index, depth
    }

    let mut words = Vec::<String>::new();
    let mut word = String::new();

    let mut state = State::Unquoted;

    for (i, grapheme) in rawline.graphemes(false).enumerate() {
        match state {
            State::Unquoted => match grapheme {
                " " | "\n" | "\t" => {
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


