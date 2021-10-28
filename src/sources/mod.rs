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
    fn is_tty(&self) -> bool; // TODO: Remove &self
}

// TODO: This should be a vec of lines, and should include sourcekind...
// Buffer is not a valid sourcekind; should be 'subshell'
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
}
