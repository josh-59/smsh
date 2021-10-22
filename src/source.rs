use anyhow::Result;
use std::io::{self, Stdin, Write};

use super::line::Line;

pub enum SourceKind {
    TTY,
}

pub trait Source {
    fn get_line(&mut self, prompt: &str) -> Result<Option<Line>>;
    fn is_tty(&self) -> bool;
}

pub struct TTY {
    stdin: Stdin,
    line_num: usize
}

impl TTY {
    pub fn new() -> Result<Box<dyn Source>> {
        let stdin = io::stdin();

        let source = Box::new(TTY{ stdin, line_num: 0});
        Ok(source)
    }
}

impl Source for TTY {
    fn get_line(&mut self, prompt: &str) -> Result<Option<Line>> {
        let mut buffer = String::new();

        print!("{}", prompt);
        io::stdout().flush()?;

        self.stdin.read_line(&mut buffer)?;
        self.line_num += 1;

        let line = Line::new(buffer, self.line_num, SourceKind::TTY)?;

        Ok(Some(line))
    }

    fn is_tty(&self) -> bool {
        true
    }
}
