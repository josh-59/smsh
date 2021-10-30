use anyhow::Result;

use crate::line::Line;
use super::InputSource;

// Used to push lines back onto the execution stack
pub struct SubshellSource {
    lines: Vec<Line>,
    line_num: usize,
}

impl SubshellSource {
    pub fn build_source(lines: Vec<Line>) -> Box<dyn InputSource> {
        Box::new(SubshellSource{ lines, line_num: 0 })
    }
}

impl InputSource for SubshellSource {
    fn get_line(&mut self, _prompt: Option<String>) -> Result<Option<Line>> {
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

