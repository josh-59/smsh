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
    raw_text: String,
    words: Vec<Word>,
    line_type: Option<LineType>,
    line_id: LineID,
    indentation: usize,
}

impl Line {
    // TODO: Line could be incomplete upon a call to this method.  It could
    // be an escaped newline, or an unmatched quote.
    pub fn new(raw_text: String, line_num: usize, source_kind: SourceKind) -> Result<Line> {
        let line_id = LineID {
            source_kind,
            line_num,
        };

        let indentation = determine_indentation(&raw_text);

        Ok(Line {
            raw_text,
            line_type: None,
            words: Vec::new(),
            line_id,
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
        Ok(())
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

    pub fn is_complete(&self) -> bool {
        #[derive(PartialEq, Eq, Clone, Copy)]
        enum State {
            SingleQuoted,
            DoubleQuoted,
            Escaped,
            FoundPipe,
            Unquoted,
        }

        let mut state = State::Unquoted;
        let mut escaped_state = State::Unquoted;
        let mut found_escaped_newline = false;

        for grapheme in self.raw_text.graphemes(true) {
            found_escaped_newline = false;

            state = match state {
                State::Unquoted => match grapheme {
                    "\'" => State::SingleQuoted,
                    "\"" => State::DoubleQuoted,
                    "\\" => {
                        escaped_state = State::Unquoted;
                        State::Escaped
                    }
                    "|" => State::FoundPipe,
                    _ => state,
                },
                State::SingleQuoted => match grapheme {
                    "\'" => State::Unquoted,
                    _ => state,
                },
                State::DoubleQuoted => match grapheme {
                    "\"" => State::Unquoted,
                    "\\" => {
                        escaped_state = State::DoubleQuoted;
                        State::Escaped
                    }
                    _ => state,
                },
                State::FoundPipe => match grapheme {
                    " " | "\t" | "\n" => state, // Ignore whitespace following pipe character
                    "\'" => State::SingleQuoted,
                    "\"" => State::DoubleQuoted,
                    _ => State::Unquoted,
                },
                State::Escaped => {
                    if grapheme == "\n" {
                        found_escaped_newline = true;
                    }

                    escaped_state
                }
            };
        }

        state == State::Unquoted && !found_escaped_newline
    }

    pub fn is_elif(&self) -> bool {
        self.line_type == Some(LineType::Elif)
    }

    pub fn is_else(&self) -> bool {
        self.line_type == Some(LineType::Else)
    }

    pub fn raw_text(&self) -> &str {
        &self.raw_text
    }

    pub fn separate(&mut self, smsh: &Shell) -> Result<()> {
        // Get parts of line
        let mut words = Vec::<Word>::new();
        for part in get_parts(&self.raw_text)? {
            words.push(Word::new(part)?);
        }

        self.words = words;

        // Line type should be stated after it's been inspected for completeness
        // and correctness.
        let line_type = if self.words.len() > 0 {
            determine_line_type(self.words[0].text())
        } else {
            LineType::Empty
        };

        Ok(())
    }

    pub fn select(&mut self) -> Result<()> {
        for word in &mut self.words {
            word.select()?;
        }

        Ok(())
    }

    pub fn source(&self) -> &SourceKind {
        &self.line_id.source_kind
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

// This function simply looks at the
fn determine_line_type(line: &str) -> LineType {
    match line {
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

// Breaks rawline into parts to be analyzed later on.
// Quotes and escapes are preserved; unquoted whitespace is removed
fn get_parts(rawline: &str) -> Result<Vec<String>> {
    #[derive(PartialEq, Eq, Clone, Copy)]
    enum State {
        SingleQuoted,
        DoubleQuoted,
        Escaped,
        Unquoted,
    }

    let mut parts = Vec::<String>::new();
    let mut part = String::new();

    let mut state = State::Unquoted;
    let mut escaped_state: Option<State> = None;

    for grapheme in rawline.graphemes(true) {
        match state {
            State::Unquoted => match grapheme {
                " " | "\t" => {
                    if !part.is_empty() {
                        parts.push(part);
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
        }
    }
    if !part.is_empty() {
        parts.push(part);
    }

    match state {
        State::SingleQuoted => Err(anyhow!("Unmatched single quote.")),
        State::DoubleQuoted => Err(anyhow!("Unmatched double quote.")),
        State::Escaped => Err(anyhow!(
            "Unresolved terminal escaped newline (This should not happen!)"
        )),
        State::Unquoted => Ok(parts),
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn determine_completeness_1() {
        let text = "echo one two three four";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(true, line.is_complete());
    }

    #[test]
    fn determine_completeness_2() {
        let text = "echo one two | cat";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(true, line.is_complete());
    }

    #[test]
    fn determine_completeness_3() {
        let text = "echo \"one two\" | cat";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(true, line.is_complete());
    }

    #[test]
    fn determine_completeness_4() {
        let text = "echo \'one two\' | cat";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(true, line.is_complete());
    }

    #[test]
    fn determine_completeness_5() {
        let text = "echo \"!{cat file}\" | cat";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(true, line.is_complete());
    }

    #[test]
    fn determine_completeness_6() {
        let text = "echo \"!{cat file}\" |";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(false, line.is_complete());
    }

    #[test]
    fn determine_completeness_7() {
        let text = "echo '!{cat file}' |     ";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(false, line.is_complete());
    }

    #[test]
    fn determine_completeness_8() {
        let text = "echo \"!{cat file} ";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(false, line.is_complete());
    }

    #[test]
    fn determine_completeness_9() {
        let text = "echo '!{cat file} ";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(false, line.is_complete());
    }

    #[test]
    fn determine_completeness_10() {
        let text = "echo '!{cat file}' | cat \\";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(false, line.is_complete());
    }

    #[test]
    fn determine_completeness_11() {
        let text = "echo '!{cat file}' | cat \\
";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(false, line.is_complete());
    } // Line ends with escaped newline -> false

    #[test]
    fn determine_completeness_12() {
        let text = "echo '!{cat file}' | cat \\  ";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(true, line.is_complete());
    } // Line ends with escaped whitespace -> true

    #[test]
    fn determine_completeness_13() {
        let text = "echo '!{cat file}' | cat \\\n";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(false, line.is_complete());
    } // Line ends with escaped newline -> false

    #[test]
    fn determine_completeness_14() {
        let text = "echo '!{cat file}' | cat \\\n echo one two three";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(true, line.is_complete());
    }

    #[test]
    fn determine_completeness_15() {
        let text = "echo '!{cat file}' | cat \\\n echo one two three\n";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(true, line.is_complete());
    }

    #[test]
    fn determine_completeness_16() {
        let text = "echo '!{cat file}' | \"";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(false, line.is_complete());
    }

    #[test]
    fn determine_completeness_17() {
        let text = "echo '!{cat file}' | \\\n\"";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(false, line.is_complete());
    }

    #[test]
    fn determine_completeness_18() {
        let text = "echo '!{cat file}' \\\n\"";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(false, line.is_complete());
    }

    #[test]
    fn determine_completeness_19() {
        let text = "echo '!{cat file}' \" \\\n\"";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(true, line.is_complete());
    }

    #[test]
    fn determine_completeness_20() {
        let text = "echo '!{cat file}\' \" \\\n\"";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(true, line.is_complete());
    }

    #[test]
    fn determine_completeness_21() {
        let text = "\\\n\\\n\\\n\\\n";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(false, line.is_complete());
    }

    #[test]
    fn determine_completeness_22() {
        let text = "\\\n\\\n\\\n\\\n ";
        let line = Line::new(text.to_string(), 1, SourceKind::Tty).unwrap();
        assert_eq!(true, line.is_complete());
    }
}
