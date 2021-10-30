use anyhow::Result;
use nix::unistd::getuid;
use std::io::{self, Stdin, Write};

use super::{InputSource, SourceKind};
use crate::line::Line;

pub struct Tty {
    stdin: Stdin,
    line_num: usize,
    last_line: Option<Line>
}

impl Tty {
    pub fn build_source() -> Box<dyn InputSource> {
        let stdin = io::stdin();

        Box::new(Tty { stdin, line_num: 0, last_line: None})
    }

    // Used to complete logical lines when they transcend physical lines
    pub fn get_secondary_line(&mut self) -> Result<Option<String>> {
        let mut buffer = String::new();

        print!("> ");
        io::stdout().flush()?;

        let num_bytes_read = self.stdin.read_line(&mut buffer)?;

        if num_bytes_read == 0 {
            Ok(None)
        } else {
            Ok(Some(buffer))
        }
    }
}

impl InputSource for Tty {

    fn get_line(&mut self, prompt: Option<String>) -> Result<Option<Line>> {
        let prompt = if let Some(p) = prompt {
            p
        } else if getuid().is_root() {
            "# ".to_string()
        } else {
            "$ ".to_string()
        };

        let mut buffer = String::new();

        print!("{}", prompt);
        io::stdout().flush()?;

        // Buffer contains newline
        let num_bytes_read = self.stdin.read_line(&mut buffer)?;

        if num_bytes_read == 0 {
            Ok(None) // EOF was found
        } else {
            self.line_num += 1;

            let mut line = Line::new(buffer, self.line_num, SourceKind::Tty);

            while !line.is_complete() {
                match self.get_secondary_line()? {
                    Some(line_addendum) => {
                        line.append(line_addendum);
                    }
                    None => {
                        eprintln!("smsh: Unexpected EOF");
                        return Ok(Some(Line::new("".to_string(), self.line_num, SourceKind::Tty)));
                    }
                }
            }

            self.last_line = Some(line.clone());

            Ok(Some(line))
        }
    }

    fn is_tty(&self) -> bool {
        true
    }

    fn is_faux_source(&self) -> bool {
        false
    }

    fn print_error(&mut self) -> Result<()> {
        if let Some(line) = &self.last_line {
            eprintln!("{}", line);
        }

        Ok(())
    }
}
