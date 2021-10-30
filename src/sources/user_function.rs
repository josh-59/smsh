use anyhow::Result;

use super::{Source, SourceKind};
use crate::line::Line;

#[derive(Clone)]
pub struct UserFunction {
    fn_name: String,
    fn_body: Vec<String>,
    line_num: usize,
}

impl UserFunction {
    pub fn new(fn_name: String, fn_body: Vec<String>) -> UserFunction {
        UserFunction {
            fn_name,
            fn_body,
            line_num: 0,
        }
    }

    pub fn build_source(self) -> Box<dyn Source> {
        Box::new(self)
    }

    pub fn name(&self) -> &str {
        &self.fn_name
    }
}

impl Source for UserFunction {
    fn get_line(&mut self, _prompt: Option<String>) -> Result<Option<Line>> {
        if self.line_num == self.fn_body.len() {
            Ok(None)
        } else {
            let text = self.fn_body[self.line_num].clone();
            self.line_num += 1;
            Ok(Some(Line::new(
                text,
                self.line_num,
                SourceKind::UserFunction(self.fn_name.clone()),
            )))
        }
    }

    fn is_tty(&self) -> bool {
        false
    }

    fn is_faux_source(&self) -> bool {
        false
    }

    fn print_error(&mut self) -> Result<()> {
        eprintln!("{}", 
                    Line::new(self.fn_body[self.line_num - 1].clone(),
                    self.line_num,
                    SourceKind::UserFunction(self.fn_name.clone())));
        Ok(())
    }
}
