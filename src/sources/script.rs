use anyhow::Result;

use super::{Source, SourceKind};
use crate::line::Line;

use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Script {
    path: PathBuf,
    body: Vec<String>,
    line_num: usize,
}

impl Script {
    pub fn new(path: PathBuf) -> Result<Box<dyn Source>> {
        let body = read_to_string(&path)?
            .lines()
            .map(|x| x.to_string())
            .collect();

        let script = Script {
            path,
            body,
            line_num: 0,
        };

        Ok(Box::new(script))
    }

    pub fn file_name(&self) -> String {
        self.path.to_string_lossy().to_string()
    }
}

impl Source for Script {
    fn get_line(&mut self, _prompt: Option<String>) -> Result<Option<Line>> {
        if self.line_num == self.body.len() {
            Ok(None)
        } else {
            let text = self.body[self.line_num].clone();
            self.line_num += 1;

            let line = Line::new(text, self.line_num, SourceKind::Script(self.file_name()));

            Ok(Some(line))
        }
    }

    fn is_tty(&self) -> bool {
        false
    }
}
