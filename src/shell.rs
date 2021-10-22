use anyhow::Result;
use super::source::{Source, TTY};
use super::line::Line;

pub struct Shell {
    sources: Vec<Box<dyn Source>>
}

impl Shell {
    pub fn new() -> Result<Shell> {
        let tty = TTY::new()?;

        let sources = vec![tty];
        Ok(Shell{ sources })
    }

    pub fn run(&mut self) -> Result<()> {
        while let Some(line) = self.get_line()? {
            println!("{}", line);
        }
        Ok(())
    }

    fn get_line(&mut self) -> Result<Option<Line>> {
        if let Some(mut source) = self.sources.pop() {
            if let Some(line) = source.get_line(">> ")? {
                self.sources.push(source);
                Ok(Some(line))
            } else {
                self.get_line()
            }
        } else {
            Ok(None)
        }
    }
}
