use anyhow::Result;
use nix::unistd::getuid;

use std::io::{self, Stdin, Write};

use super::{Source, SourceKind, Prompt, is_complete};
use crate::line::Line;

pub struct Tty {
    stdin: Stdin,
    line_num: usize,
    last_line: Option<Line>
}

impl Tty {
    pub fn build_source() -> Box<dyn Source> {
        let stdin = io::stdin();

        Box::new(Tty { stdin, line_num: 0, last_line: None})
    }

    // Used to complete logical lines when they transcend physical lines
    fn get_secondary_line(&mut self) -> Result<Option<String>> {
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

    fn simple_prompt() -> String {
        if getuid().is_root() {
            "# ".to_string()
        } else {
            "$ ".to_string()
        }
    }
}

impl Source for Tty {
    fn get_line(&mut self, prompt: Prompt) -> Result<Option<Line>> {
        let prompt = match prompt {
            Prompt::MainLoop => {
                Tty::simple_prompt()
            }
            Prompt::Block => {
                "> ".to_string()
            }
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

            while !is_complete(buffer.as_str()) {
                match self.get_secondary_line()? {
                    Some(line_addendum) => {
                        buffer.push_str(line_addendum.as_str());
                    }
                    None => {
                        eprintln!("smsh: Unexpected EOF");
                        return Ok(Some(Line::new("".to_string(), self.line_num, SourceKind::Tty)?));
                    }
                }
            }

            let line = Line::new(buffer, self.line_num, SourceKind::Tty)?;

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
