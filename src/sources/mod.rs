// TODO:
//      Implement Display for each Source, so that backtrace can
//      be neater
//
//      Sources::sources could be a vec of tuples (Box<dyn Source>, SourceKind)
use std::collections::VecDeque;

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
    fn is_faux_source(&self) -> bool;
    fn print_error(&mut self) -> Result<()>;
}

pub struct Sources {
    sources: Vec<Box<dyn Source>>,
    buffer: VecDeque<Line>,
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
    // Blocks are delimited by a single blank line.
    pub fn get_block(&mut self, source_kind: &SourceKind, indent: usize) -> Result<Vec<Line>> {
        let mut lines = Vec::<Line>::new();

        if !self.buffer.is_empty() {
            while let Some(line) = self.buffer.pop_front() {
                if *line.source() == *source_kind && line.indentation() == indent {
                    lines.push(line);
                } else {
                    self.buffer.push_front(line);
                    break;
                }
            }
        } else if let Some(mut source) = self.sources.pop() {
            while let Some(line) = source.get_line()? {
                if *line.source() == *source_kind && line.indentation() == indent {
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

    pub fn push_front(&mut self, line: Line) {
        self.buffer.push_front(line);
    }

    pub fn push_back(&mut self, line: Line) {
        self.buffer.push_back(line);
    }

    pub fn push_source(&mut self, source: Box<dyn Source>) {
        self.sources.push(source)
    }

    pub fn clear_sources(&mut self) {
        self.sources.clear();
    }

    pub fn backtrace(&mut self) {
        while let Some(mut source) = self.sources.pop() {
            if !source.is_faux_source() && !source.is_tty() {
                let _ = source.print_error();
            }

            if source.is_tty() {
                self.sources.push(source);
                break;
            }
        }
    }
}
