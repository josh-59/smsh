use anyhow::Result;

use crate::line::Line;
use super::{Source, SourceKind};

#[derive(Clone)]
pub struct UserFunction {
    fn_name: String,
    fn_body: Vec<String>,
    line_num: usize
}

impl UserFunction {
    pub fn new(fn_name: String, fn_body: Vec<String>) -> UserFunction {
        UserFunction { fn_name, fn_body, line_num: 0 }
    }

    pub fn build_source(&self) -> Box<dyn Source> {
        let source = Box::new(self.clone());
        source
    }
}

impl Source for UserFunction {
    fn get_line(&mut self, _prompt: &str) -> Result<Option<Line>> {
        if self.line_num == self.fn_body.len() {
            Ok(None)
        } else {
            let text = self.fn_body[self.line_num].clone();
            self.line_num += 1;
            Ok(Some(Line::new(text, self.line_num, SourceKind::UserFunction(self.fn_name.clone()))))
        }
    }

    fn is_tty(&self) -> bool {
        false 
    }
}
