use anyhow::Result;
use std::io::{self, Stdin, Write};

use super::line::Line;

pub trait Source {
    fn get_line(&mut self, prompt: &str) -> Result<Option<Line>>;
}

pub struct TTY {
    stdin: Stdin,
}

impl TTY {
    pub fn new() -> Result<Box<dyn Source>> {
        let stdin = io::stdin();

        let source = Box::new( TTY { stdin } );
        Ok( source )
    }
}

impl Source for TTY {
    fn get_line(&mut self, prompt: &str) -> Result<Option<Line>> {
        let mut buffer = String::new();

        print!("{}", prompt);
        io::stdout().flush()?;

        self.stdin.read_line(&mut buffer)?;

        let line = Line::new(buffer)?;

        Ok(Some(line))
    }
}
