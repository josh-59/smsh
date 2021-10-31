use anyhow::Result;

use crate::line::Line;
use super::{Source, Prompt};

// Used to push lines back onto the execution stack
pub struct SubshellSource {
    lines: Vec<Line>,
    line_num: usize,
}

impl SubshellSource {
    pub fn build_source(lines: Vec<Line>) -> Box<dyn Source> {
        Box::new(SubshellSource{ lines, line_num: 0 })
    }
}

impl Source for SubshellSource {
    fn get_line(&mut self, _prompt: Prompt) -> Result<Option<Line>> {
        if self.line_num == self.lines.len() {
            Ok(None)
        } else {
            self.line_num += 1;
            Ok(Some(self.lines[self.line_num - 1].clone()))
        }
    }

    fn is_tty(&self) -> bool {
        false
    }

    fn is_faux_source(&self) -> bool {
        false
    }

    fn print_error(&mut self) -> Result<()> {
        if self.line_num > 0 {
            eprintln!("{}", self.lines[self.line_num - 1]);
        }

        Ok(())
    }
}

