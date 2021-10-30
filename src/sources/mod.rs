// TODO: Get rid of 'BufferSource altogether'.
//       It clutters up the code and also clutters up runtime,
//       making debugging more difficult.
//       Should instead have a function, 
//       'push_back(&mut self, line: Line)'
//       defined in Source.
//
//       Create proper Subshell Source for subshell expansion
//
//       Implement Display for each Source, so that backtrace can
//       be neater
//
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

pub trait InputSource {
    fn get_line(&mut self, prompt: Option<String>) -> Result<Option<Line>>;
    fn is_tty(&self) -> bool; 
    fn is_faux_source(&self) -> bool;
    fn print_error(&mut self) -> Result<()>;
}

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

