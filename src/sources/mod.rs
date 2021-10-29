use anyhow::Result;

use super::line::Line;

pub mod script;
pub mod tty;
pub mod user_function;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum SourceKind {
    Tty,
    Subshell,
    UserFunction(String), // String contains function name
    Script(String),       // String contains script pathname
}

pub trait Source {
    fn get_line(&mut self, prompt: Option<String>) -> Result<Option<Line>>;
    fn is_tty(&self) -> bool; 
    fn is_faux_source(&self) -> bool;
    fn print_error(&mut self) -> Result<()>;
}

// Used to push lines back onto the execution stack
pub struct BufferSource {
    lines: Vec<Line>,
    line_num: usize,
}

impl BufferSource {
    pub fn build_source(lines: Vec<Line>) -> Box<dyn Source> {
        Box::new(BufferSource { lines, line_num: 0 })
    }
}

impl Source for BufferSource {
    fn get_line(&mut self, _prompt: Option<String>) -> Result<Option<Line>> {
        if self.line_num == self.lines.len() {
            Ok(None)
        } else {
            self.line_num += 1;
            Ok(Some(self.lines[self.line_num - 1].clone()))
        }
    }

    fn is_tty(&self) -> bool {
        if self.lines.is_empty() {
            false
        } else {
            *self.lines[0].source() == SourceKind::Tty
        }
    }

    fn is_faux_source(&self) -> bool {
        if !self.lines.is_empty() {
            match self.lines[0].source() {
                SourceKind::Tty | 
                SourceKind::Subshell |
                SourceKind::UserFunction(_) |
                SourceKind::Script(_) => {
                    false
                }
                _ => {
                    true
                }
            }
        } else {
            true
        }
    }

    fn print_error(&mut self) -> Result<()> {
        if self.line_num > 0 {
            eprintln!("{}", self.lines[self.line_num - 1]);
        }

        Ok(())
    }
}

