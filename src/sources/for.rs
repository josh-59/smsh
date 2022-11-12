use anyhow::Result;

use super::Source;
use crate::line::{Line, LineID};

#[derive(Clone)]
pub struct For {
    iterator_key: String,
    iterator_values: Vec<String>,
    body: Vec<Line>,
    line_num: usize,
    iter_idx: usize,
    for_line_identifier: LineID, // Topmost line identifier
}

impl For {
    pub fn new(iterator_key: String, iterator_values: Vec<String>, body: Vec<Line>, for_line_identifier: LineID) -> Self{
        Self {
            iterator_key,
            iterator_values,
            body,
            line_num: 0,
            iter_idx: 0,
            for_line_identifier,
        }
    }

    pub fn build_source(self) -> Box<dyn Source> {
        Box::new(self)
    }
}

impl Source for For {
    fn get_line(&mut self) -> Result<Option<Line>> {

        // At the top of each loop, execute let statement
        if self.line_num == 0 { 
            if self.iter_idx == self.iterator_values.len() {
                Ok(None)
            } else {
                let rawline = format!("let {} = {}", 
                                      self.iterator_key, 
                                      self.iterator_values[self.iter_idx]);
                self.iter_idx += 1;

                let line = Line::new(rawline, 
                             self.for_line_identifier.line_num, 
                             self.for_line_identifier.source_kind.clone())?;
                
                self.line_num += 1;

                Ok(Some(line))
            }
        } else { // self.line_num is now one greater than true index
            if self.body.is_empty() {
                Ok(None)
            } else {
                let line = self.body[self.line_num - 1].clone();

                self.line_num += 1;
                if self.line_num - 1 == self.body.len() {
                    self.line_num = 0;
                }

                Ok(Some(line))
            }

        }
    }

    fn is_tty(&self) -> bool {
        false
    }

    fn print_error(&mut self) -> Result<()> {
        Ok(())
    }
}
