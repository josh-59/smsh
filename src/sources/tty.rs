use anyhow::Result;
use nix::unistd::getuid;
use std::io::{self, Stdin, Write};

use crate::line::Line;
use super::{Source, SourceKind};

pub struct TTY {
    stdin: Stdin,
    line_num: usize
}

impl TTY {
    pub fn new() -> Box<dyn Source> {
        let stdin = io::stdin();

        Box::new(TTY{ stdin, line_num: 0})
    }

    // Used to complete logical lines when they transcend physical lines
    pub fn get_secondary_line(&mut self) -> Result<String> {

        let mut buffer = String::new();

        print!("> ");
        io::stdout().flush()?;

        self.stdin.read_line(&mut buffer)?;

        Ok(buffer)
    }
}

impl Source for TTY {
    fn get_line(&mut self, prompt: Option<String>) -> Result<Option<Line>> {
        let prompt = if let Some(p) = prompt {
            p
        } else {
            if getuid().is_root() {
                "# ".to_string()
            } else {
                "$ ".to_string()
            }
        };


        let mut buffer = String::new();

        print!("{}", prompt);
        io::stdout().flush()?;

        self.stdin.read_line(&mut buffer)?;
        self.line_num += 1;

        let mut line = Line::new(buffer, self.line_num, SourceKind::TTY);

        while !line.is_complete() {
            line.append(self.get_secondary_line()?)
        }

        Ok(Some(line))
    }

    fn is_tty(&self) -> bool {
        true
    }
}
