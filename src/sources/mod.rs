// TODO: Get rid of 'BufferSource altogether'.
//       It clutters up the code and also clutters up runtime,
//       making debugging more difficult.
//       Should instead have a function, 
//       'push_back(&mut self, line: Line)'
//       defined in Source.
//
//       Create proper Subshell Source for subshell expansion
//
//       Implement Display for each Source, so that backtrace can
//       be neater
//
use anyhow::Result;

use super::line::Line;

pub mod script;
pub mod tty;
pub mod user_function;
pub mod subshell;
pub mod r#for;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum SourceKind {
    Tty,
    Subshell,
    UserFunction(String), // String contains function name
    Script(String),       // String contains script pathname
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Prompt {
    Block,
    MainLoop,
}

pub trait Source {
    fn get_line(&mut self, prompt: Prompt) -> Result<Option<Line>>;
    fn is_tty(&self) -> bool; 
    fn is_faux_source(&self) -> bool;
    fn print_error(&mut self) -> Result<()>;
}

pub struct Sources {
    sources: Vec<Box<dyn Source>>,
    buffer: Vec<Line>,
}

impl Sources {
    pub fn new() -> Self {
        Sources { sources: vec![], buffer: vec![] }
    }

    pub fn get_line(&mut self, prompt: Prompt ) -> Result<Option<Line>> {
        if let Some(line) = self.buffer.pop() {
            Ok(Some(line))
        } else if let Some(mut source) = self.sources.pop() {
            match source.get_line(prompt) { 
                Ok(Some(line)) => {
                    self.sources.push(source);
                    Ok(Some(line))
                }
                Ok(None) => {
                    self.get_line(prompt)
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

        if let Some(mut source) = self.sources.pop() {
            while let Some(line) = source.get_line(Prompt::Block)? {
                if *line.source() == *source_kind && line.indentation() == indent {
                    lines.push(line);
            } else {
                    self.sources.push(source);
                    self.buffer.push(line);
                    break;
                }
            }
        }

        Ok(lines)
    }

    pub fn push_back(&mut self, line: Line) {
        self.buffer.push(line);
    }

    // Pushes block of lines onto buffer.
    // Order is preserved.
    pub fn push_block(&mut self, lines: Vec<Line>) {
        for line in lines.iter().rev() {
            self.buffer.push(line.clone());
        }
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



// True if line is a complete logical line
pub fn is_complete(rawline: &str) -> bool {
    let mut single_quoted = false;
    let mut double_quoted = false;
    let mut escaped = false;

    for ch in rawline.chars() {
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

