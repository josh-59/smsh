use anyhow::Result;

use super::line::Line;

pub mod tty;
pub mod user_function;

pub enum SourceKind {
    TTY,
    Buffer, 
    UserFunction(String), // String contains function name
}

pub trait Source {
    fn get_line(&mut self, prompt: &str) -> Result<Option<Line>>;
    fn is_tty(&self) -> bool; // TODO: Remove &self
}

pub struct BufferSource {
    lines: Vec<String>,
    line_num: usize,
}

impl BufferSource {
    pub fn new(lines: Vec<String>) -> Box<dyn Source> {
        Box::new(BufferSource { lines, line_num: 0 })
    }
}

impl Source for BufferSource {
    fn get_line(&mut self, _prompt: &str) -> Result<Option<Line>> {
        if self.line_num == self.lines.len() {
            Ok(None)
        } else {
            self.line_num += 1;
            let line = Line::new(self.lines[self.line_num - 1].clone(),
                                 self.line_num, 
                                 SourceKind::Buffer);
            Ok(Some(line))
        }
    }

    fn is_tty(&self) -> bool {
        false
    }
}
