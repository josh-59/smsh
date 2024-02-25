// TODO:
//      Implement Display for each Source, so that backtrace can
//      be neater
//
//      Sources::sources could be a vec of tuples (Box<dyn Source>, SourceKind)
use std::collections::VecDeque;

use unicode_segmentation::UnicodeSegmentation;

use anyhow::Result;

use super::line::Line;

pub mod script;
pub mod tty;
pub mod user_function;
pub mod subshell;
pub mod r#for;

// Used in Line struct to identify source
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum SourceKind {
    Tty,
    Subshell,
    UserFunction(String), // String contains function name
    Script(String),       // String contains script pathname
}

pub trait Source {
    fn get_line(&mut self) -> Result<Option<Line>>;
    fn is_tty(&self) -> bool; 
    fn print_error(&mut self) -> Result<()>;
}

pub struct Sources {
    sources: Vec<Box<dyn Source>>,
    buffer: VecDeque<Line>, // Only contains complete lines
}

impl Sources {
    pub fn new() -> Self {
        Sources { sources: vec![], buffer: VecDeque::<Line>::new()}
    }

    pub fn get_line(&mut self) -> Result<Option<Line>> {
        if let Some(line) = self.buffer.pop_front() {
            Ok(Some(line))
        } else if let Some(mut source) = self.sources.pop() {
            match source.get_line() { 
                Ok(Some(line)) => {
                    self.sources.push(source);
                    Ok(Some(line))
                }
                Ok(None) => {
                    self.get_line()
                }
                Err(e) => {
                    self.sources.push(source);
                    Err(e)
                }
            }
        } else {
            Ok(None)
        }
    }

    // Captures one block from a single source
    pub fn get_block(&mut self, source_kind: &SourceKind, indent: usize) -> Result<Vec<Line>> {
        let mut lines = Vec::<Line>::new();

        if !self.buffer.is_empty() {
            while let Some(line) = self.buffer.pop_front() {
                if *line.source() == *source_kind && line.indentation() >= indent {
                    lines.push(line);
                } else {
                    self.buffer.push_front(line);
                    break;
                }
            }
        } else if let Some(mut source) = self.sources.pop() {
            while let Some(line) = source.get_line()? {
                if *line.source() == *source_kind && line.indentation() >= indent {
                    lines.push(line);
            } else {
                    self.sources.push(source);
                    self.buffer.push_front(line);
                    break;
                }
            }
        } 

        Ok(lines)
    }

    // Pushes block of lines onto buffer.
    // Order is preserved.
    pub fn push_block(&mut self, lines: Vec<Line>) {
        for line in lines {
            self.buffer.push_back(line);
        }
    }

    pub fn push_line(&mut self, line: Line) {
        self.buffer.push_front(line);
    }

    pub fn push_source(&mut self, source: Box<dyn Source>) {
        self.sources.push(source)
    }

    pub fn clear(&mut self) {
        self.sources.clear();
    }

    pub fn backtrace(&mut self) {
        while let Some(mut source) = self.sources.pop() {
            if source.is_tty() {
                self.sources.push(source);
                break;
            } else {
                let _ = source.print_error();
            }
        }
    }
}


// Determines if `text` is a complete logical line.  Ignores newlines
fn is_complete(text: &str) -> bool {
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

    for grapheme in text.graphemes(true) {
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


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn determine_completeness_1() {
        let text = "echo one two three four";
        assert!(is_complete(text))
    }

    #[test]
    fn determine_completeness_2() {
        let text = "echo one two | cat";
        assert!(is_complete(text))
    }

    #[test]
    fn determine_completeness_3() {
        let text = "echo \"one two\" | cat";
        assert!(is_complete(text))
    }

    #[test]
    fn determine_completeness_4() {
        let text = "echo \'one two\' | cat";
        assert!(is_complete(text))
    }

    #[test]
    fn determine_completeness_5() {
        let text = "echo \"!{cat file}\" | cat";
        assert!(is_complete(text))
    }

    #[test]
    fn determine_completeness_6() {
        let text = "echo \"!{cat file}\" |";
        assert!(!is_complete(text))
    }

    #[test]
    fn determine_completeness_7() {
        let text = "echo '!{cat file}' |     ";
        assert!(!is_complete(text))
    }

    #[test]
    fn determine_completeness_8() {
        let text = "echo \"!{cat file} ";
        assert!(!is_complete(text))
    }

    #[test]
    fn determine_completeness_9() {
        let text = "echo '!{cat file} ";
        assert!(!is_complete(text))
    }

    #[test]
    fn determine_completeness_10() {
        let text = "echo '!{cat file}' | cat \\";
        assert!(!is_complete(text))
    }

    #[test]
    fn determine_completeness_11() {
        let text = "echo '!{cat file}' | cat \\
";
        assert!(!is_complete(text))
    } // Line ends with escaped newline -> false

    #[test]
    fn determine_completeness_12() {
        let text = "echo '!{cat file}' | cat \\  ";
        assert!(is_complete(text))
    } // Line ends with escaped whitespace -> true

    #[test]
    fn determine_completeness_13() {
        let text = "echo '!{cat file}' | cat \\\n";
        assert!(!is_complete(text))
    } // Line ends with escaped newline -> false

    #[test]
    fn determine_completeness_14() {
        let text = "echo '!{cat file}' | cat \\\n echo one two three";
        assert!(is_complete(text))
    }

    #[test]
    fn determine_completeness_15() {
        let text = "echo '!{cat file}' | cat \\\n echo one two three\n";
        assert!(is_complete(text))
    }

    #[test]
    fn determine_completeness_16() {
        let text = "echo '!{cat file}' | \"";
        assert!(!is_complete(text))
    }

    #[test]
    fn determine_completeness_17() {
        let text = "echo '!{cat file}' | \\\n\"";
        assert!(!is_complete(text))
    }

    #[test]
    fn determine_completeness_18() {
        let text = "echo '!{cat file}' \\\n\"";
        assert!(!is_complete(text))
    }

    #[test]
    fn determine_completeness_19() {
        let text = "echo '!{cat file}' \" \\\n\"";
        assert!(is_complete(text))
    }

    #[test]
    fn determine_completeness_20() {
        let text = "echo '!{cat file}\' \" \\\n\"";
        assert!(is_complete(text))
    }

    #[test]
    fn determine_completeness_21() {
        let text = "\\\n\\\n\\\n\\\n";
        assert!(!is_complete(text))
    }

    #[test]
    fn determine_completeness_22() {
        let text = "\\\n\\\n\\\n\\\n ";
        assert!(is_complete(text))
    }
}
